//! Modelo mínimo e independente do DOM para compor interfaces Patina.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use patina_reactive::Signal;

/// Texto calculado sob demanda. O renderer o reavalia quando os signals
/// lidos dentro dele mudam.
#[derive(Clone)]
pub struct DynText(Rc<dyn Fn() -> String>);

impl DynText {
    pub fn new(f: impl Fn() -> String + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub fn get(&self) -> String {
        (self.0)()
    }
}

impl fmt::Debug for DynText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DynText(..)")
    }
}

impl PartialEq for DynText {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Valor de atributo: fixo ou recalculado quando os signals mudam.
#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    Static(String),
    Dynamic(DynText),
}

impl AttrValue {
    pub fn get(&self) -> String {
        match self {
            AttrValue::Static(value) => value.clone(),
            AttrValue::Dynamic(text) => text.get(),
        }
    }
}

/// Um atributo HTML.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttrValue,
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self { name: name.into(), value: AttrValue::Static(value.into()) }
    }

    pub fn dynamic(name: impl Into<String>, f: impl Fn() -> String + 'static) -> Self {
        Self { name: name.into(), value: AttrValue::Dynamic(DynText::new(f)) }
    }
}

/// Evento entregue a um listener. O renderer guarda o evento nativo em `raw`
/// (no navegador, um `web_sys::Event`) e, para campos de formulário, o valor
/// atual do campo em `value`.
pub struct Event {
    raw: Box<dyn Any>,
    value: Option<String>,
    prevent_default: Option<Box<dyn Fn()>>,
}

impl Event {
    pub fn new<T: 'static>(raw: T) -> Self {
        Self { raw: Box::new(raw), value: None, prevent_default: None }
    }

    pub fn with_value(mut self, value: Option<String>) -> Self {
        self.value = value;
        self
    }

    /// Usado pelo renderer para ligar `prevent_default` ao evento nativo.
    pub fn with_prevent_default(mut self, f: impl Fn() + 'static) -> Self {
        self.prevent_default = Some(Box::new(f));
        self
    }

    /// Cancela a ação padrão do navegador (por exemplo, enviar um `<form>`).
    pub fn prevent_default(&self) {
        if let Some(f) = &self.prevent_default {
            f();
        }
    }

    /// O evento nativo, se for do tipo `T`.
    pub fn raw<T: 'static>(&self) -> Option<&T> {
        self.raw.downcast_ref()
    }

    /// Valor do `<input>`, `<textarea>` ou `<select>` que disparou o evento.
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event").field("value", &self.value).finish_non_exhaustive()
    }
}

/// Tipo de evento com nome conhecido, usado em `.on(Click, ...)`.
pub trait EventType {
    const NAME: &'static str;
}

/// Eventos do DOM mais comuns. Para outros, use `ElementBuilder::on_event`.
pub mod ev {
    macro_rules! event_types {
        ($($name:ident => $dom:literal),* $(,)?) => {
            $(
                #[doc = concat!("Evento `", $dom, "`.")]
                #[derive(Debug, Clone, Copy)]
                pub struct $name;

                impl super::EventType for $name {
                    const NAME: &'static str = $dom;
                }
            )*
        };
    }

    event_types! {
        Click => "click",
        DoubleClick => "dblclick",
        Input => "input",
        Change => "change",
        Submit => "submit",
        KeyDown => "keydown",
        KeyUp => "keyup",
        Focus => "focus",
        Blur => "blur",
    }
}

/// Um listener ligado a um elemento.
#[derive(Clone)]
pub struct Listener {
    pub event: String,
    handler: Rc<RefCell<dyn FnMut(Event)>>,
}

impl Listener {
    pub fn new(event: impl Into<String>, handler: impl FnMut(Event) + 'static) -> Self {
        Self { event: event.into(), handler: Rc::new(RefCell::new(handler)) }
    }

    /// Entrega `event` ao handler.
    pub fn call(&self, event: Event) {
        (self.handler.borrow_mut())(event);
    }
}

impl fmt::Debug for Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Listener").field("event", &self.event).finish_non_exhaustive()
    }
}

impl PartialEq for Listener {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && Rc::ptr_eq(&self.handler, &other.handler)
    }
}

type ViewFn = Rc<dyn Fn() -> View>;

/// Conteúdo condicional criado por [`show`].
#[derive(Clone)]
pub struct Show {
    when: Rc<dyn Fn() -> bool>,
    then: ViewFn,
    otherwise: Option<ViewFn>,
}

impl Show {
    /// Conteúdo mostrado quando a condição é falsa.
    pub fn otherwise(mut self, f: impl Fn() -> View + 'static) -> Self {
        self.otherwise = Some(Rc::new(f));
        self
    }

    /// Avalia a condição, inscrevendo o effect atual.
    pub fn condition(&self) -> bool {
        (self.when)()
    }

    /// Constrói o ramo para a condição dada; `None` quando não há conteúdo.
    pub fn branch(&self, condition: bool) -> Option<View> {
        if condition {
            Some((self.then)())
        } else {
            self.otherwise.as_ref().map(|f| f())
        }
    }
}

impl fmt::Debug for Show {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Show").field("otherwise", &self.otherwise.is_some()).finish_non_exhaustive()
    }
}

impl PartialEq for Show {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.when, &other.when)
    }
}

impl From<Show> for View {
    fn from(show: Show) -> Self {
        View::Show(show)
    }
}

/// Mostra `then()` enquanto `when()` for verdadeiro. O conteúdo é criado de
/// novo (num escopo próprio) só quando a condição muda.
///
/// `show(move || logado.get(), || el("p").text("Bem-vindo")).otherwise(|| el("p").text("Entre"))`
pub fn show(when: impl Fn() -> bool + 'static, then: impl Fn() -> View + 'static) -> Show {
    Show { when: Rc::new(when), then: Rc::new(then), otherwise: None }
}

trait KeyValue {
    fn as_any(&self) -> &dyn Any;
    fn eq_key(&self, other: &dyn KeyValue) -> bool;
    fn hash_key(&self, state: &mut dyn Hasher);
}

impl<K: Hash + Eq + 'static> KeyValue for K {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_key(&self, other: &dyn KeyValue) -> bool {
        other.as_any().downcast_ref::<K>() == Some(self)
    }

    fn hash_key(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }
}

/// Chave de um item de lista, de qualquer tipo `Hash + Eq`.
#[derive(Clone)]
pub struct Key(Rc<dyn KeyValue>);

impl Key {
    pub fn new<K: Hash + Eq + 'static>(key: K) -> Self {
        Key(Rc::new(key))
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_key(&*other.0)
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_key(state);
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key(..)")
    }
}

/// Um item de lista: a chave e como construir a view dele.
pub struct ListItem {
    pub key: Key,
    build: Box<dyn FnOnce() -> View>,
}

impl ListItem {
    pub fn build(self) -> View {
        (self.build)()
    }
}

/// Lista com chave criada por [`each`].
#[derive(Clone)]
pub struct List {
    items: Rc<dyn Fn() -> Vec<ListItem>>,
}

impl List {
    /// Lê os itens atuais, inscrevendo o effect atual. As views ainda não
    /// foram construídas: só as chaves.
    pub fn items(&self) -> Vec<ListItem> {
        (self.items)()
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("List(..)")
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.items, &other.items)
    }
}

/// Lista com chave. Quando `items()` muda, só os itens com chave nova são
/// construídos, os removidos são descartados e os que ficaram mantêm os
/// mesmos nós do DOM (mesmo se mudarem de posição). Chaves repetidas: só a
/// primeira ocorrência é mostrada.
///
/// `each(move || tarefas.get(), |t| t.id, |t| el("li").text(t.texto).build())`
pub fn each<T, K, I>(
    items: impl Fn() -> I + 'static,
    key: impl Fn(&T) -> K + 'static,
    render: impl Fn(T) -> View + 'static,
) -> View
where
    I: IntoIterator<Item = T>,
    T: 'static,
    K: Hash + Eq + 'static,
{
    let render = Rc::new(render);
    View::List(List {
        items: Rc::new(move || {
            items()
                .into_iter()
                .map(|item| {
                    let key = Key::new(key(&item));
                    let render = render.clone();
                    ListItem { key, build: Box::new(move || render(item)) }
                })
                .collect()
        }),
    })
}

/// Árvore de visualização usada pelo renderer.
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Text(String),
    /// Texto que muda quando os signals lidos mudam; só este nó é atualizado.
    Dynamic(DynText),
    /// Conteúdo condicional; ver [`show`].
    Show(Show),
    /// Lista com chave; ver [`each`].
    List(List),
    Fragment(Vec<View>),
    Element {
        tag: String,
        attributes: Vec<Attribute>,
        listeners: Vec<Listener>,
        children: Vec<View>,
    },
}

impl View {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Texto recalculado: `View::text_with(move || format!("Olá, {}", nome.get()))`.
    pub fn text_with(f: impl Fn() -> String + 'static) -> Self {
        Self::Dynamic(DynText::new(f))
    }

    pub fn fragment<I>(children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<View>,
    {
        Self::Fragment(children.into_iter().map(Into::into).collect())
    }

    pub fn element(tag: impl Into<String>) -> ElementBuilder {
        ElementBuilder {
            tag: tag.into(),
            attributes: Vec::new(),
            listeners: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Serializa o estado atual em HTML, avaliando o texto dinâmico uma vez.
    /// Feito para testes e depuração; não é renderização no servidor.
    pub fn render_html(&self) -> String {
        let mut out = String::new();
        self.write_html(&mut out);
        out
    }

    fn write_html(&self, out: &mut String) {
        match self {
            View::Text(value) => escape_into(value, out),
            View::Dynamic(text) => escape_into(&text.get(), out),
            View::Show(show) => {
                if let Some(branch) = show.branch(show.condition()) {
                    branch.write_html(out);
                }
            }
            View::List(list) => {
                let mut seen = std::collections::HashSet::new();
                for item in list.items() {
                    if seen.insert(item.key.clone()) {
                        item.build().write_html(out);
                    }
                }
            }
            View::Fragment(children) => children.iter().for_each(|child| child.write_html(out)),
            View::Element { tag, attributes, children, .. } => {
                out.push('<');
                out.push_str(tag);
                for attribute in attributes {
                    out.push(' ');
                    out.push_str(&attribute.name);
                    out.push_str("=\"");
                    escape_into(&attribute.value.get(), out);
                    out.push('"');
                }
                out.push('>');
                if VOID_ELEMENTS.contains(&tag.as_str()) {
                    return;
                }
                children.iter().for_each(|child| child.write_html(out));
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
            }
        }
    }
}

const VOID_ELEMENTS: &[&str] = &["area", "br", "col", "hr", "img", "input", "link", "meta", "source", "wbr"];

fn escape_into(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

/// Atalho para `View::element`: `el("main").class("pagina")`.
pub fn el(tag: impl Into<String>) -> ElementBuilder {
    View::element(tag)
}

impl From<&str> for View {
    fn from(value: &str) -> Self {
        View::text(value)
    }
}

impl From<String> for View {
    fn from(value: String) -> Self {
        View::Text(value)
    }
}

/// Um signal como filho vira um nó de texto atualizado sozinho.
impl<T: fmt::Display + 'static> From<Signal<T>> for View {
    fn from(signal: Signal<T>) -> Self {
        View::text_with(move || signal.with(ToString::to_string))
    }
}

/// Unidade reutilizável de interface: uma struct com os dados e `view()`.
pub trait Component {
    fn view(self) -> View;
}

impl<C: Component> From<C> for View {
    fn from(component: C) -> Self {
        component.view()
    }
}

/// Builder ergonômico para uma view de elemento.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementBuilder {
    tag: String,
    attributes: Vec<Attribute>,
    listeners: Vec<Listener>,
    children: Vec<View>,
}

impl ElementBuilder {
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(name, value));
        self
    }

    /// Atributo recalculado quando os signals lidos em `f` mudam.
    pub fn attr_with(mut self, name: impl Into<String>, f: impl Fn() -> String + 'static) -> Self {
        self.attributes.push(Attribute::dynamic(name, f));
        self
    }

    /// Atalho para `attr("class", value)`.
    pub fn class(self, value: impl Into<String>) -> Self {
        self.attr("class", value)
    }

    /// Acrescenta um nó de texto como filho.
    pub fn text(self, value: impl Into<String>) -> Self {
        self.child(View::text(value))
    }

    /// Acrescenta um nó de texto recalculado como filho.
    pub fn text_with(self, f: impl Fn() -> String + 'static) -> Self {
        self.child(View::text_with(f))
    }

    /// Liga um handler a um evento conhecido: `.on(Click, move |_| ...)`.
    pub fn on<E: EventType>(self, _event: E, handler: impl FnMut(Event) + 'static) -> Self {
        self.on_event(E::NAME, handler)
    }

    /// Liga um handler a um evento pelo nome do DOM.
    pub fn on_event(mut self, event: impl Into<String>, handler: impl FnMut(Event) + 'static) -> Self {
        self.listeners.push(Listener::new(event, handler));
        self
    }

    /// Aceita uma `View`, um `ElementBuilder` ainda não finalizado, texto,
    /// um `Signal` ou um `Component`.
    pub fn child(mut self, child: impl Into<View>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children<I>(mut self, children: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<View>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> View {
        View::Element {
            tag: self.tag,
            attributes: self.attributes,
            listeners: self.listeners,
            children: self.children,
        }
    }
}

impl From<ElementBuilder> for View {
    fn from(builder: ElementBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::ev::{Click, Input};
    use super::{each, el, show, Attribute, Component, Event, Key, View};
    use patina_reactive::signal;

    #[test]
    fn builds_an_element_with_attributes_and_children() {
        let view = View::element("h1")
            .attr("class", "title")
            .child(View::text("Olá"))
            .build();

        assert_eq!(
            view,
            View::Element {
                tag: "h1".to_string(),
                attributes: vec![Attribute::new("class", "title")],
                listeners: vec![],
                children: vec![View::text("Olá")],
            }
        );
    }

    #[test]
    fn fragment_keeps_children_in_order() {
        let view = View::fragment([View::text("A"), View::text("B")]);
        assert_eq!(view, View::Fragment(vec![View::text("A"), View::text("B")]));
    }

    #[test]
    fn el_shortcuts_match_the_explicit_builder() {
        let short = el("main")
            .class("pagina")
            .child(el("h1").text("Olá, Patina!"))
            .child("rodapé")
            .build();

        let explicit = View::element("main")
            .attr("class", "pagina")
            .child(View::element("h1").child(View::text("Olá, Patina!")).build())
            .child(View::text("rodapé"))
            .build();

        assert_eq!(short, explicit);
    }

    #[test]
    fn children_accepts_mixed_into_view_items() {
        let view = el("ul").children(["um", "dois"]).build();
        assert_eq!(view.render_html(), "<ul>umdois</ul>");
    }

    #[test]
    fn render_html_escapes_text_and_attributes() {
        let view = el("p").attr("title", "a \"b\"").text("<x> & y").build();
        assert_eq!(view.render_html(), "<p title=\"a &quot;b&quot;\">&lt;x&gt; &amp; y</p>");
        assert_eq!(el("input").attr("value", "1").build().render_html(), "<input value=\"1\">");
    }

    fn first_listener(view: &View) -> super::Listener {
        match view {
            View::Element { listeners, .. } => listeners[0].clone(),
            _ => panic!("esperava um elemento"),
        }
    }

    #[test]
    fn signal_child_and_click_listener_update_the_text() {
        let cliques = signal(0);
        let botao = el("button")
            .on(Click, move |_| cliques.update(|n| *n += 1))
            .text("Cliques: ")
            .child(cliques)
            .build();

        assert_eq!(botao.render_html(), "<button>Cliques: 0</button>");
        let listener = first_listener(&botao);
        assert_eq!(listener.event, "click");
        listener.call(Event::new(()));
        listener.call(Event::new(()));
        assert_eq!(botao.render_html(), "<button>Cliques: 2</button>");
    }

    #[test]
    fn input_event_carries_the_field_value() {
        let nome = signal(String::new());
        let campo = el("input")
            .on(Input, move |ev| nome.set(ev.value().unwrap_or_default().to_string()))
            .build();
        let saudacao = el("p").text_with(move || format!("Olá, {}!", nome.get())).build();

        first_listener(&campo).call(Event::new(()).with_value(Some("Ana".into())));
        assert_eq!(saudacao.render_html(), "<p>Olá, Ana!</p>");
    }

    #[test]
    fn dynamic_attribute_follows_its_signal() {
        let ativo = signal(false);
        let view = el("div")
            .attr_with("class", move || if ativo.get() { "ativo".into() } else { String::new() })
            .build();
        ativo.set(true);
        assert_eq!(view.render_html(), "<div class=\"ativo\"></div>");
    }

    struct Cartao {
        titulo: String,
    }

    impl Component for Cartao {
        fn view(self) -> View {
            el("article").child(el("h2").text(self.titulo)).build()
        }
    }

    #[test]
    fn show_follows_its_condition() {
        let logado = signal(false);
        let view = el("div")
            .child(show(move || logado.get(), || el("p").text("Bem-vindo").build()).otherwise(|| "Entre".into()))
            .build();
        assert_eq!(view.render_html(), "<div>Entre</div>");
        logado.set(true);
        assert_eq!(view.render_html(), "<div><p>Bem-vindo</p></div>");
    }

    #[test]
    fn each_renders_items_in_order_and_skips_duplicate_keys() {
        let itens = signal(vec![(1, "um"), (2, "dois"), (1, "repetido")]);
        let view = el("ul")
            .child(each(move || itens.get(), |item| item.0, |item| el("li").text(item.1).build()))
            .build();
        assert_eq!(view.render_html(), "<ul><li>um</li><li>dois</li></ul>");
    }

    #[test]
    fn keys_compare_by_type_and_value() {
        assert_eq!(Key::new(1u32), Key::new(1u32));
        assert_ne!(Key::new(1u32), Key::new(2u32));
        assert_ne!(Key::new(1u32), Key::new(1u64), "tipos diferentes nunca são iguais");
        assert_eq!(Key::new(String::from("a")), Key::new(String::from("a")));
    }

    #[test]
    fn prevent_default_calls_the_renderer_hook() {
        let chamado = std::rc::Rc::new(std::cell::Cell::new(false));
        let flag = chamado.clone();
        let event = Event::new(()).with_prevent_default(move || flag.set(true));
        event.prevent_default();
        assert!(chamado.get());
        Event::new(()).prevent_default(); // sem renderer: não faz nada
    }

    #[test]
    fn components_are_children_like_any_view() {
        let view = el("section").child(Cartao { titulo: "Um".into() }).build();
        assert_eq!(view.render_html(), "<section><article><h2>Um</h2></article></section>");
    }
}
