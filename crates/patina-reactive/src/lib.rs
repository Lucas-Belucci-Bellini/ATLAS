//! Reatividade mínima do Patina: signals, effects e escopos.
//!
//! Tudo vive num runtime por thread. Um [`Signal`] é um identificador `Copy`
//! para um valor guardado no runtime; ler o signal dentro de um [`effect`]
//! inscreve o effect, e escrever no signal roda de novo os effects inscritos.
//!
//! Signals e effects criados dentro de [`create_root`] pertencem àquele escopo
//! e são liberados por [`Root::dispose`], que também roda as funções
//! registradas com [`on_cleanup`]. Fora de qualquer escopo, eles vivem até o
//! fim da thread.
//!
//! Effects criados *durante* a execução de outro effect pertencem ao escopo
//! atual, não ao effect. Quem precisa trocar conteúdo (o renderer, em `show`
//! e listas) cria um escopo filho com [`with_owner`] + [`create_root`] e o
//! descarta antes de renderizar de novo.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

const DISPOSED: &str = "patina: signal usado depois que o seu escopo foi descartado";
const TYPE_MISMATCH: &str = "patina: tipo do signal não confere (bug interno)";

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct NodeId {
    index: usize,
    generation: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ScopeId {
    index: usize,
    generation: u32,
}

type EffectFn = Rc<RefCell<dyn FnMut()>>;

enum NodeKind {
    Signal {
        value: Rc<RefCell<dyn Any>>,
        subscribers: Vec<NodeId>,
    },
    Effect {
        run: EffectFn,
        sources: Vec<NodeId>,
    },
}

struct Slot {
    generation: u32,
    kind: Option<NodeKind>,
}

#[derive(Default)]
struct ScopeData {
    parent: Option<ScopeId>,
    nodes: Vec<NodeId>,
    children: Vec<ScopeId>,
    cleanups: Vec<Box<dyn FnOnce()>>,
}

struct ScopeSlot {
    generation: u32,
    data: Option<ScopeData>,
}

#[derive(Default)]
struct Runtime {
    slots: Vec<Slot>,
    free: Vec<usize>,
    scopes: Vec<ScopeSlot>,
    free_scopes: Vec<usize>,
    owner: Option<ScopeId>,
    observer: Option<NodeId>,
    batch_depth: usize,
    pending: Vec<NodeId>,
}

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default());
}

/// Acesso exclusivo ao runtime. Nunca chame código do usuário aqui dentro:
/// colete o que for preciso e execute depois de soltar o empréstimo.
fn with_rt<R>(f: impl FnOnce(&mut Runtime) -> R) -> R {
    RUNTIME.with(|rt| f(&mut rt.borrow_mut()))
}

impl Runtime {
    fn insert(&mut self, kind: NodeKind) -> NodeId {
        let id = if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index];
            slot.kind = Some(kind);
            NodeId { index, generation: slot.generation }
        } else {
            self.slots.push(Slot { generation: 0, kind: Some(kind) });
            NodeId { index: self.slots.len() - 1, generation: 0 }
        };
        if let Some(scope) = self.owner.and_then(|owner| self.scope_mut(owner)) {
            scope.nodes.push(id);
        }
        id
    }

    fn get(&self, id: NodeId) -> Option<&NodeKind> {
        self.slots
            .get(id.index)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.kind.as_ref())
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut NodeKind> {
        self.slots
            .get_mut(id.index)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.kind.as_mut())
    }

    /// Remove o nó e desfaz as inscrições. O valor removido é devolvido para
    /// ser destruído fora do empréstimo do runtime.
    fn remove(&mut self, id: NodeId) -> Option<NodeKind> {
        let slot = self.slots.get_mut(id.index)?;
        if slot.generation != id.generation {
            return None;
        }
        let kind = slot.kind.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);

        match &kind {
            NodeKind::Signal { subscribers, .. } => {
                for effect in subscribers {
                    if let Some(NodeKind::Effect { sources, .. }) = self.get_mut(*effect) {
                        sources.retain(|source| *source != id);
                    }
                }
            }
            NodeKind::Effect { sources, .. } => {
                for source in sources {
                    if let Some(NodeKind::Signal { subscribers, .. }) = self.get_mut(*source) {
                        subscribers.retain(|effect| *effect != id);
                    }
                }
            }
        }
        Some(kind)
    }

    fn track(&mut self, signal: NodeId) {
        let Some(effect) = self.observer else { return };
        let mut subscribed = false;
        if let Some(NodeKind::Signal { subscribers, .. }) = self.get_mut(signal) {
            if !subscribers.contains(&effect) {
                subscribers.push(effect);
                subscribed = true;
            }
        }
        if subscribed {
            if let Some(NodeKind::Effect { sources, .. }) = self.get_mut(effect) {
                sources.push(signal);
            }
        }
    }

    fn scope_mut(&mut self, id: ScopeId) -> Option<&mut ScopeData> {
        self.scopes
            .get_mut(id.index)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.data.as_mut())
    }

    fn new_scope(&mut self) -> ScopeId {
        let parent = self.owner.filter(|owner| self.scope_mut(*owner).is_some());
        let data = ScopeData { parent, ..ScopeData::default() };
        let id = if let Some(index) = self.free_scopes.pop() {
            let slot = &mut self.scopes[index];
            slot.data = Some(data);
            ScopeId { index, generation: slot.generation }
        } else {
            self.scopes.push(ScopeSlot { generation: 0, data: Some(data) });
            ScopeId { index: self.scopes.len() - 1, generation: 0 }
        };
        if let Some(parent) = parent.and_then(|parent| self.scope_mut(parent)) {
            parent.children.push(id);
        }
        id
    }

    fn take_scope(&mut self, id: ScopeId) -> Option<ScopeData> {
        let slot = self.scopes.get_mut(id.index)?;
        if slot.generation != id.generation {
            return None;
        }
        let data = slot.data.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free_scopes.push(id.index);
        Some(data)
    }
}

/// Estado reativo. É `Copy`: pode ser movido para vários closures.
pub struct Signal<T> {
    id: NodeId,
    _value: PhantomData<fn() -> T>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({}:{})", self.id.index, self.id.generation)
    }
}

/// Cria um signal com o valor inicial.
pub fn signal<T: 'static>(value: T) -> Signal<T> {
    let value: Rc<RefCell<dyn Any>> = Rc::new(RefCell::new(value));
    let id = with_rt(|rt| rt.insert(NodeKind::Signal { value, subscribers: Vec::new() }));
    Signal { id, _value: PhantomData }
}

impl<T: 'static> Signal<T> {
    fn cell(&self, track: bool) -> Rc<RefCell<dyn Any>> {
        with_rt(|rt| {
            if track {
                rt.track(self.id);
            }
            match rt.get(self.id) {
                Some(NodeKind::Signal { value, .. }) => value.clone(),
                _ => panic!("{DISPOSED}"),
            }
        })
    }

    /// Lê o valor por referência, inscrevendo o effect atual.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let cell = self.cell(true);
        let guard = cell.borrow();
        let value = guard.downcast_ref::<T>().expect(TYPE_MISMATCH);
        f(value)
    }

    /// Copia o valor, inscrevendo o effect atual.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(T::clone)
    }

    /// Copia o valor sem inscrever o effect atual.
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        untrack(|| self.get())
    }

    /// Substitui o valor e notifica os effects inscritos.
    pub fn set(&self, value: T) {
        self.update(|current| *current = value);
    }

    /// Altera o valor no lugar e notifica os effects inscritos.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let cell = self.cell(false);
        {
            let mut guard = cell.borrow_mut();
            f(guard.downcast_mut::<T>().expect(TYPE_MISMATCH));
        }
        notify(self.id);
    }

    /// `false` depois que o escopo dono do signal foi descartado.
    pub fn is_alive(&self) -> bool {
        with_rt(|rt| rt.get(self.id).is_some())
    }
}

fn notify(signal: NodeId) {
    let to_run = with_rt(|rt| {
        let subscribers = match rt.get(signal) {
            Some(NodeKind::Signal { subscribers, .. }) => subscribers.clone(),
            _ => return Vec::new(),
        };
        if rt.batch_depth > 0 {
            for effect in subscribers {
                if !rt.pending.contains(&effect) {
                    rt.pending.push(effect);
                }
            }
            Vec::new()
        } else {
            subscribers
        }
    });
    for effect in to_run {
        run_effect(effect);
    }
}

fn run_effect(id: NodeId) {
    let prepared = with_rt(|rt| {
        let run = match rt.get(id) {
            Some(NodeKind::Effect { run, .. }) => run.clone(),
            _ => return None,
        };
        // Um effect que escreve num signal que ele mesmo lê não roda de novo
        // dentro de si; mantém as inscrições atuais.
        if run.try_borrow_mut().is_err() {
            return None;
        }
        if let Some(NodeKind::Effect { sources, .. }) = rt.get_mut(id) {
            let sources = std::mem::take(sources);
            for source in sources {
                if let Some(NodeKind::Signal { subscribers, .. }) = rt.get_mut(source) {
                    subscribers.retain(|effect| *effect != id);
                }
            }
        }
        let previous = rt.observer.replace(id);
        Some((run, previous))
    });
    let Some((run, previous)) = prepared else { return };
    (run.borrow_mut())();
    with_rt(|rt| rt.observer = previous);
}

/// Identificador de um effect, para descartá-lo antes do seu escopo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Effect {
    id: NodeId,
}

impl Effect {
    /// Para o effect: ele não roda mais.
    pub fn dispose(self) {
        let removed = with_rt(|rt| rt.remove(self.id));
        drop(removed);
    }
}

/// Roda `f` agora e de novo sempre que um signal lido por ele mudar.
pub fn effect(f: impl FnMut() + 'static) -> Effect {
    let run: EffectFn = Rc::new(RefCell::new(f));
    let id = with_rt(|rt| rt.insert(NodeKind::Effect { run, sources: Vec::new() }));
    run_effect(id);
    Effect { id }
}

/// Agrupa escritas: cada effect afetado roda uma única vez, no fim.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    with_rt(|rt| rt.batch_depth += 1);
    let result = f();
    let pending = with_rt(|rt| {
        rt.batch_depth -= 1;
        if rt.batch_depth == 0 {
            std::mem::take(&mut rt.pending)
        } else {
            Vec::new()
        }
    });
    for effect in pending {
        run_effect(effect);
    }
    result
}

/// Executa `f` sem inscrever o effect atual nos signals lidos.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    let previous = with_rt(|rt| rt.observer.take());
    let result = f();
    with_rt(|rt| rt.observer = previous);
    result
}

/// O escopo que será dono do que for criado agora. Guarde-o para criar
/// escopos filhos mais tarde, de dentro de um effect.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Owner {
    id: ScopeId,
}

impl Owner {
    /// O escopo atual, se houver.
    pub fn current() -> Option<Owner> {
        with_rt(|rt| rt.owner.map(|id| Owner { id }))
    }
}

/// Executa `f` com `owner` como escopo atual (`None`: nenhum escopo).
/// Um `Owner` já descartado não adota nada.
pub fn with_owner<R>(owner: Option<Owner>, f: impl FnOnce() -> R) -> R {
    let previous = with_rt(|rt| std::mem::replace(&mut rt.owner, owner.map(|owner| owner.id)));
    let result = f();
    with_rt(|rt| rt.owner = previous);
    result
}

/// Escopo dono de signals, effects e limpezas. Não é descartado no `drop`:
/// chame [`Root::dispose`].
#[derive(Debug)]
pub struct Root {
    id: ScopeId,
}

/// Cria um escopo, executa `f` dentro dele e devolve o resultado e o escopo.
/// Um escopo criado dentro de outro é descartado junto com o pai.
pub fn create_root<R>(f: impl FnOnce() -> R) -> (R, Root) {
    let id = with_rt(Runtime::new_scope);
    let result = with_owner(Some(Owner { id }), f);
    (result, Root { id })
}

impl Root {
    /// O escopo como `Owner`, para criar filhos depois.
    pub fn owner(&self) -> Owner {
        Owner { id: self.id }
    }

    /// Executa `f` com este escopo como dono do que for criado.
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        with_owner(Some(self.owner()), f)
    }

    /// Roda as limpezas (das mais recentes para as mais antigas, filhos antes
    /// do pai) e libera todos os signals, effects e escopos filhos.
    pub fn dispose(self) {
        let (cleanups, removed) = with_rt(|rt| {
            let parent = rt.scope_mut(self.id).and_then(|data| data.parent);
            if let Some(parent) = parent.and_then(|parent| rt.scope_mut(parent)) {
                parent.children.retain(|child| *child != self.id);
            }

            let mut cleanups = Vec::new();
            let mut removed = Vec::new();
            let mut stack = vec![self.id];
            while let Some(id) = stack.pop() {
                let Some(data) = rt.take_scope(id) else { continue };
                stack.extend(data.children);
                cleanups.extend(data.cleanups);
                removed.extend(data.nodes.into_iter().filter_map(|node| rt.remove(node)));
            }
            (cleanups, removed)
        });
        for cleanup in cleanups.into_iter().rev() {
            cleanup();
        }
        drop(removed);
    }
}

/// Registra `f` para rodar quando o escopo atual for descartado. Fora de um
/// escopo, `f` nunca roda (e o que ele captura continua vivo).
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    let leftover = with_rt(|rt| match rt.owner.and_then(|owner| rt.scope_mut(owner)) {
        Some(scope) => {
            scope.cleanups.push(Box::new(f));
            None
        }
        None => Some(f),
    });
    if let Some(f) = leftover {
        std::mem::forget(f);
    }
}

/// Quantidade de signals e effects vivos na thread atual. Útil em testes de
/// vazamento.
#[doc(hidden)]
pub fn live_nodes() -> usize {
    with_rt(|rt| rt.slots.iter().filter(|slot| slot.kind.is_some()).count())
}

/// Quantidade de escopos vivos na thread atual. Útil em testes de vazamento.
#[doc(hidden)]
pub fn live_scopes() -> usize {
    with_rt(|rt| rt.scopes.iter().filter(|slot| slot.data.is_some()).count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn get_set_update() {
        let n = signal(1);
        assert_eq!(n.get(), 1);
        n.set(2);
        n.update(|v| *v *= 10);
        assert_eq!(n.get(), 20);
    }

    #[test]
    fn effect_reruns_only_for_signals_it_read() {
        let a = signal(0);
        let b = signal(0);
        let runs = Rc::new(Cell::new(0));
        let counter = runs.clone();
        effect(move || {
            a.get();
            counter.set(counter.get() + 1);
        });
        assert_eq!(runs.get(), 1);
        a.set(1);
        assert_eq!(runs.get(), 2);
        b.set(1);
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn dependencies_follow_the_branch_taken() {
        let use_a = signal(true);
        let a = signal("a");
        let b = signal("b");
        let seen = Rc::new(RefCell::new(Vec::new()));
        let log = seen.clone();
        effect(move || {
            let value = if use_a.get() { a.get() } else { b.get() };
            log.borrow_mut().push(value);
        });
        use_a.set(false);
        a.set("a2"); // não é mais dependência
        b.set("b2");
        assert_eq!(*seen.borrow(), vec!["a", "b", "b2"]);
    }

    #[test]
    fn batch_runs_each_effect_once() {
        let a = signal(0);
        let b = signal(0);
        let runs = Rc::new(Cell::new(0));
        let counter = runs.clone();
        effect(move || {
            a.get();
            b.get();
            counter.set(counter.get() + 1);
        });
        batch(|| {
            a.set(1);
            b.set(1);
        });
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn untrack_does_not_subscribe() {
        let a = signal(0);
        let runs = Rc::new(Cell::new(0));
        let counter = runs.clone();
        effect(move || {
            a.get_untracked();
            counter.set(counter.get() + 1);
        });
        a.set(1);
        assert_eq!(runs.get(), 1);
    }

    #[test]
    fn effect_writing_its_own_dependency_does_not_loop() {
        let n = signal(0);
        effect(move || {
            if n.get() < 3 {
                n.update(|v| *v += 1);
            }
        });
        assert_eq!(n.get_untracked(), 1);
        n.set(1);
        assert_eq!(n.get_untracked(), 2);
    }

    #[test]
    fn dispose_runs_cleanups_and_frees_everything() {
        let outside = signal(0);
        let cleaned = Rc::new(Cell::new(false));
        let flag = cleaned.clone();
        let runs = Rc::new(Cell::new(0));
        let counter = runs.clone();
        let before = live_nodes();

        let (inner, root) = create_root(move || {
            let inner = signal(0);
            effect(move || {
                outside.get();
                inner.get();
                counter.set(counter.get() + 1);
            });
            create_root(|| on_cleanup(move || flag.set(true)));
            inner
        });

        assert_eq!(live_nodes(), before + 2);
        root.dispose();
        assert!(cleaned.get(), "limpeza do escopo filho deve rodar");
        assert!(!inner.is_alive());
        assert_eq!(live_nodes(), before, "nada pode sobrar depois do dispose");
        assert_eq!(live_scopes(), 0);

        outside.set(1);
        assert_eq!(runs.get(), 1, "effect descartado não roda");
    }

    #[test]
    #[should_panic(expected = "descartado")]
    fn reading_a_disposed_signal_panics() {
        let (n, root) = create_root(|| signal(0));
        root.dispose();
        n.get();
    }

    #[test]
    fn child_scopes_created_from_an_effect_are_swapped_without_leaks() {
        let toggle = signal(0);
        let ((), parent) = create_root(|| {
            let owner = Owner::current();
            let mut current: Option<Root> = None;
            effect(move || {
                toggle.get();
                if let Some(old) = current.take() {
                    old.dispose();
                }
                let ((), child) = untrack(|| {
                    with_owner(owner, || {
                        create_root(|| {
                            signal(());
                        })
                    })
                });
                current = Some(child);
            });
        });

        let nodes = live_nodes();
        for i in 1..50 {
            toggle.set(i);
        }
        assert_eq!(live_nodes(), nodes, "cada troca descarta o escopo anterior");
        assert_eq!(live_scopes(), 2, "pai + o filho atual");

        parent.dispose();
        assert_eq!(live_scopes(), 0);
    }

    #[test]
    fn a_disposed_owner_does_not_adopt_into_a_reused_scope() {
        let ((), first) = create_root(|| ());
        let stale = first.owner();
        first.dispose();

        let ((), second) = create_root(|| ()); // reaproveita o índice
        let orphan = with_owner(Some(stale), || signal(1));
        second.dispose();

        assert!(orphan.is_alive(), "o signal não pertence ao escopo novo");
    }
}
