use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use patina_core::{AttrValue, Event, Key, List, Listener, Show, View};
use patina_reactive::{create_root, effect, on_cleanup, untrack, with_owner, Owner, Root};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, EventTarget, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Node};

/// Uma árvore montada. `unmount` remove os nós, os listeners e os effects.
/// Soltar o `Mounted` sem chamar `unmount` mantém a interface viva.
#[derive(Debug)]
pub struct Mounted {
    root: Root,
    element: Element,
}

impl Mounted {
    pub fn unmount(self) {
        self.root.dispose();
        clear(&self.element);
    }
}

/// Monta a árvore criada por `build` no primeiro elemento que casa com o
/// seletor CSS `target` (por exemplo `"#app"`), substituindo os filhos atuais.
/// `build` roda dentro do escopo da montagem: signals criados nele são
/// liberados no `unmount`.
pub fn mount(target: &str, build: impl FnOnce() -> View) -> Result<Mounted, JsValue> {
    let result = try_mount(target, build);
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    result
}

fn try_mount(target: &str, build: impl FnOnce() -> View) -> Result<Mounted, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window indisponível"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document indisponível"))?;
    let element = document
        .query_selector(target)?
        .ok_or_else(|| JsValue::from_str(&format!("nenhum elemento casa com o seletor `{target}`")))?;

    let (rendered, root) = create_root(|| render_node(&document, &build()));
    match rendered {
        Ok(node) => {
            clear(&element);
            element.append_child(&node)?;
            Ok(Mounted { root, element })
        }
        Err(error) => {
            root.dispose();
            Err(error)
        }
    }
}

fn clear(element: &Element) {
    while let Some(child) = element.first_child() {
        let _ = element.remove_child(&child);
    }
}

fn render_node(document: &Document, view: &View) -> Result<Node, JsValue> {
    match view {
        View::Text(value) => Ok(document.create_text_node(value).unchecked_into::<Node>()),
        View::Dynamic(text) => {
            let node = document.create_text_node("");
            let target = node.clone();
            let text = text.clone();
            effect(move || target.set_data(&text.get()));
            Ok(node.unchecked_into::<Node>())
        }
        View::Show(show) => render_show(document, show),
        View::List(list) => render_list(document, list),
        View::Fragment(children) => {
            let fragment = document.create_document_fragment();
            for child in children {
                fragment.append_child(&render_node(document, child)?)?;
            }
            Ok(fragment.unchecked_into::<Node>())
        }
        View::Element { tag, attributes, listeners, children } => {
            let element = document.create_element(tag)?;
            for attribute in attributes {
                match &attribute.value {
                    AttrValue::Static(value) => element.set_attribute(&attribute.name, value)?,
                    AttrValue::Dynamic(text) => bind_attribute(&element, &attribute.name, text.clone()),
                }
            }
            for listener in listeners {
                attach(&element, listener)?;
            }
            for child in children {
                element.append_child(&render_node(document, child)?)?;
            }
            Ok(element.unchecked_into::<Node>())
        }
    }
}

/// Atributo dinâmico. Em campos de formulário, `value` também atualiza a
/// propriedade, senão o que o usuário digitou continuaria na tela.
fn bind_attribute(element: &Element, name: &str, text: patina_core::DynText) {
    let target = element.clone();
    let name = name.to_string();
    effect(move || {
        let value = text.get();
        if let Err(error) = target.set_attribute(&name, &value) {
            web_sys::console::error_1(&error);
        }
        if name == "value" {
            set_field_value(&target, &value);
        }
    });
}

fn set_field_value(element: &Element, value: &str) {
    if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
        if input.value() != value {
            input.set_value(value);
        }
    } else if let Some(area) = element.dyn_ref::<HtmlTextAreaElement>() {
        if area.value() != value {
            area.set_value(value);
        }
    } else if let Some(select) = element.dyn_ref::<HtmlSelectElement>() {
        select.set_value(value);
    }
}

/// Registra o listener e agenda a remoção para quando o escopo for descartado.
fn attach(element: &Element, listener: &Listener) -> Result<(), JsValue> {
    let handler = listener.clone();
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |raw: web_sys::Event| {
        let value = raw.target().and_then(field_value);
        let native = raw.clone();
        handler.call(
            Event::new(raw)
                .with_value(value)
                .with_prevent_default(move || native.prevent_default()),
        );
    });

    let target: EventTarget = element.clone().unchecked_into();
    target.add_event_listener_with_callback(&listener.event, closure.as_ref().unchecked_ref())?;

    let name = listener.event.clone();
    on_cleanup(move || {
        let _ = target.remove_event_listener_with_callback(&name, closure.as_ref().unchecked_ref());
        drop(closure);
    });
    Ok(())
}

fn field_value(target: EventTarget) -> Option<String> {
    let target = match target.dyn_into::<HtmlInputElement>() {
        Ok(input) => return Some(input.value()),
        Err(target) => target,
    };
    let target = match target.dyn_into::<HtmlTextAreaElement>() {
        Ok(area) => return Some(area.value()),
        Err(target) => target,
    };
    target.dyn_into::<HtmlSelectElement>().ok().map(|select| select.value())
}

// ---------------------------------------------------------------------------
// Regiões dinâmicas: conteúdo entre dois comentários-marcadores.
// ---------------------------------------------------------------------------

/// Um trecho contínuo de nós irmãos, do primeiro ao último (inclusive).
#[derive(Clone)]
struct Range {
    first: Node,
    last: Node,
}

impl Range {
    fn nodes(&self) -> Vec<Node> {
        let mut nodes = vec![self.first.clone()];
        let mut current = self.first.clone();
        while !current.is_same_node(Some(&self.last)) {
            match current.next_sibling() {
                Some(next) => {
                    nodes.push(next.clone());
                    current = next;
                }
                None => break,
            }
        }
        nodes
    }

    fn remove(&self) {
        for node in self.nodes() {
            if let Some(parent) = node.parent_node() {
                let _ = parent.remove_child(&node);
            }
        }
    }

    fn move_before(&self, before: &Node) -> Result<(), JsValue> {
        let parent = before
            .parent_node()
            .ok_or_else(|| JsValue::from_str("marcador de região sem pai"))?;
        for node in self.nodes() {
            parent.insert_before(&node, Some(before))?;
        }
        Ok(())
    }
}

/// Renderiza `view` num escopo filho de `owner`, sem inscrever o effect atual,
/// e insere o resultado antes de `before`. Devolve o trecho criado e o escopo.
fn render_range(document: &Document, owner: Option<Owner>, view: impl FnOnce() -> View, before: &Node) -> Result<(Range, Root), JsValue> {
    let (rendered, root) = untrack(|| with_owner(owner, || create_root(|| render_node(document, &view()))));
    let node = match rendered {
        Ok(node) => node,
        Err(error) => {
            root.dispose();
            return Err(error);
        }
    };

    // Um fragmento some ao ser inserido: guardamos o primeiro e o último
    // filho. Fragmento vazio ganha um comentário para ter posição.
    if node.node_type() == Node::DOCUMENT_FRAGMENT_NODE && node.first_child().is_none() {
        node.append_child(&document.create_comment(""))?;
    }
    let range = if node.node_type() == Node::DOCUMENT_FRAGMENT_NODE {
        Range { first: node.first_child().unwrap_throw(), last: node.last_child().unwrap_throw() }
    } else {
        Range { first: node.clone(), last: node.clone() }
    };

    let parent = before
        .parent_node()
        .ok_or_else(|| JsValue::from_str("marcador de região sem pai"))?;
    parent.insert_before(&node, Some(before))?;
    Ok((range, root))
}

/// Fragmento `<!--início--> <!--fim-->`, devolvido junto com o marcador final.
fn markers(document: &Document, name: &str) -> Result<(Node, Node), JsValue> {
    let fragment = document.create_document_fragment();
    fragment.append_child(&document.create_comment(name))?;
    let end: Node = document.create_comment(&format!("/{name}")).unchecked_into();
    fragment.append_child(&end)?;
    Ok((fragment.unchecked_into(), end))
}

fn render_show(document: &Document, show: &Show) -> Result<Node, JsValue> {
    let (fragment, end) = markers(document, "show")?;
    let owner = Owner::current();
    let document = document.clone();
    let show = show.clone();
    let mut shown: Option<bool> = None;
    let mut current: Option<(Range, Root)> = None;

    effect(move || {
        let condition = show.condition();
        if shown == Some(condition) {
            return;
        }
        shown = Some(condition);

        if let Some((range, root)) = current.take() {
            root.dispose();
            range.remove();
        }
        let show = show.clone();
        let Some(branch) = untrack(|| show.branch(condition)) else { return };
        match render_range(&document, owner, move || branch, &end) {
            Ok(rendered) => current = Some(rendered),
            Err(error) => web_sys::console::error_1(&error),
        }
    });
    Ok(fragment)
}

struct Row {
    key: Key,
    range: Range,
    root: Root,
}

fn render_list(document: &Document, list: &List) -> Result<Node, JsValue> {
    let (fragment, end) = markers(document, "each")?;
    let owner = Owner::current();
    let document = document.clone();
    let list = list.clone();
    let rows: Rc<RefCell<Vec<Row>>> = Rc::new(RefCell::new(Vec::new()));

    // Ao descartar o escopo dono da lista, os escopos das linhas já são
    // descartados como filhos; aqui só soltamos os identificadores.
    let cleanup_rows = rows.clone();
    on_cleanup(move || cleanup_rows.borrow_mut().clear());

    effect(move || {
        let items = list.items();
        if let Err(error) = reconcile(&document, owner, &end, &rows, items) {
            web_sys::console::error_1(&error);
        }
    });
    Ok(fragment)
}

fn reconcile(
    document: &Document,
    owner: Option<Owner>,
    end: &Node,
    rows: &RefCell<Vec<Row>>,
    items: Vec<patina_core::ListItem>,
) -> Result<(), JsValue> {
    let mut old: HashMap<Key, Row> = rows.borrow_mut().drain(..).map(|row| (row.key.clone(), row)).collect();
    let mut seen = HashSet::new();
    let mut next = Vec::with_capacity(items.len());
    let mut fresh = Vec::new();

    for item in items {
        if !seen.insert(item.key.clone()) {
            web_sys::console::warn_1(&JsValue::from_str("patina: chave repetida em each(); item ignorado"));
            continue;
        }
        match old.remove(&item.key) {
            Some(row) => next.push(Some(row)),
            None => {
                fresh.push((next.len(), item));
                next.push(None);
            }
        }
    }

    // 1. Remove o que saiu da lista.
    for (_, row) in old {
        row.root.dispose();
        row.range.remove();
    }

    // 2. Cria os itens novos (no fim; o passo 3 os coloca no lugar).
    for (index, item) in fresh {
        let key = item.key.clone();
        let (range, root) = render_range(document, owner, move || item.build(), end)?;
        next[index] = Some(Row { key, range, root });
    }

    // 3. Percorre na ordem nova com um cursor: só move o que está fora do
    //    lugar, então itens que ficaram parados não saem do DOM.
    let next: Vec<Row> = next.into_iter().flatten().collect();
    let start = first_region_node(&next, end);
    let mut cursor = start;
    for row in &next {
        if row.range.first.is_same_node(Some(&cursor)) {
            cursor = row.range.last.next_sibling().unwrap_or_else(|| end.clone());
        } else {
            row.range.move_before(&cursor)?;
        }
    }

    *rows.borrow_mut() = next;
    Ok(())
}

/// O primeiro nó depois do marcador inicial da lista.
fn first_region_node(rows: &[Row], end: &Node) -> Node {
    // O marcador inicial é o irmão anterior ao primeiro nó da região. Como
    // não guardamos o marcador inicial, voltamos a partir do fim.
    let mut node = end.clone();
    let row_nodes: HashSet<u32> = HashSet::new();
    let _ = row_nodes;
    while let Some(previous) = node.previous_sibling() {
        if previous.node_type() == Node::COMMENT_NODE && previous.text_content().as_deref() == Some("each") {
            return previous.next_sibling().unwrap_or_else(|| end.clone());
        }
        node = previous;
    }
    rows.first().map(|row| row.range.first.clone()).unwrap_or_else(|| end.clone())
}
