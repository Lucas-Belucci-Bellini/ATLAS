//! ATLAS demonstration with reactive UI and a live TITAN API request.

use patina::prelude::*;

fn contador() -> AtlasView {
    let cliques = signal(0);

    el("button")
        .class("botao")
        .on(AtlasClick, move |_| cliques.update(|n| *n += 1))
        .text("Cliques: ")
        .child(cliques)
        .build()
}

fn saudacao() -> AtlasView {
    let nome = signal(String::from("mundo"));

    el("div")
        .class("campo")
        .child(el("label").attr("for", "nome").text("Seu nome"))
        .child(
            el("input")
                .attr("id", "nome")
                .attr("value", "mundo")
                .on(AtlasInput, move |ev| {
                    nome.set(ev.value().unwrap_or_default().to_string())
                }),
        )
        .child(el("p").text_with(move || format!("Olá, {}!", nome.get())))
        .build()
}

#[cfg(target_arch = "wasm32")]
fn integracao_titan() -> AtlasView {
    use patina::http::{describe_js_error, AtlasApiEnvelope, AtlasHttpClient};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::spawn_local;

    let status = signal(String::from(
        "API ainda não consultada. Inicie o TITAN e clique no botão.",
    ));
    let status_for_click = status.clone();
    let client = AtlasHttpClient::new("http://127.0.0.1:3000").with_timeout_ms(5_000);
    let error_status_for_click = status.clone();
    let error_client = client.clone();

    let button = el("button")
        .class("botao")
        .on(AtlasClick, move |_| {
            status_for_click.set("Consultando TITAN…".to_string());

            let client = client.clone();
            let status_for_request = status_for_click.clone();
            spawn_local(async move {
                match client.get_json("/api/v1/hello").await {
                    Ok(response) => {
                        let decoded = (|| -> Result<String, JsValue> {
                            let envelope = AtlasApiEnvelope::from_js(&response.body)?;
                            if envelope.api_version != "v1" {
                                return Err(JsValue::from_str("versão de API não suportada"));
                            }

                            let message = envelope.string_field("message")?;
                            let environment = envelope.string_field("environment")?;
                            let request_id =
                                response.request_id.as_deref().unwrap_or("não informado");

                            Ok(format!(
                                "{message} (ambiente: {environment}; HTTP {}; request ID: {request_id})",
                                response.status
                            ))
                        })();

                        match decoded {
                            Ok(message) => status_for_request.set(message),
                            Err(error) => status_for_request.set(format!(
                                "Resposta fora do contrato esperado: {}",
                                describe_js_error(&error)
                            )),
                        }
                    }
                    Err(error) => status_for_request.set(format!(
                        "Falha ao consultar TITAN: {}",
                        describe_js_error(&error)
                    )),
                }
            });
        })
        .text("Consultar TITAN");

    // Intentionally call a missing versioned route to demonstrate the v1 error contract.
    let error_button = el("button")
        .class("botao")
        .on(AtlasClick, move |_| {
            error_status_for_click.set("Testando resposta de erro v1…".to_string());

            let client = error_client.clone();
            let status_for_request = error_status_for_click.clone();
            spawn_local(async move {
                match client.get_json("/api/v1/not-found").await {
                    Ok(response) => status_for_request.set(format!(
                        "Esperava HTTP 404, mas recebi HTTP {}.",
                        response.status
                    )),
                    Err(error) => status_for_request.set(format!(
                        "Erro v1 interpretado pelo cliente: {}",
                        describe_js_error(&error)
                    )),
                }
            });
        })
        .text("Testar erro v1");

    el("section")
        .class("integracao")
        .child(el("h2").text("Integração com TITAN"))
        .child(el("p").text(
            "Consulta real à API versionada por HTTP/JSON, com timeout e validação do envelope.",
        ))
        .child(button)
        .child(error_button)
        .child(el("p").attr("aria-live", "polite").child(status))
        .build()
}

pub fn pagina() -> AtlasView {
    let page =
        el("main")
            .class("atlas-demo")
            .child(el("h1").text("ATLAS — demonstração"))
            .child(el("p").text(
                "Interface reativa em Rust, preparada para consumir contratos HTTP versionados.",
            ))
            .child(contador())
            .child(saudacao());

    #[cfg(target_arch = "wasm32")]
    let page = page.child(integracao_titan());

    page.build()
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static MOUNTED_DEMO: std::cell::RefCell<Option<patina::AtlasMounted>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    let mounted = patina::mount_scoped("#app", pagina)?;
    MOUNTED_DEMO.with(|slot| *slot.borrow_mut() = Some(mounted));
    Ok(())
}

/// Removes the demo mount. Exported so the browser smoke test can exercise
/// listener and reactive-effect cleanup through the public mounted handle.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn unmount_demo() -> bool {
    MOUNTED_DEMO.with(|slot| match slot.borrow_mut().take() {
        Some(mounted) => {
            mounted.unmount();
            true
        }
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagina_renderiza_estado_inicial() {
        let html = pagina().render_html();
        assert!(html.starts_with("<main"));
        assert!(html.contains("Cliques: 0"));
        assert!(html.contains("<p>Olá, mundo!</p>"));
    }
}
