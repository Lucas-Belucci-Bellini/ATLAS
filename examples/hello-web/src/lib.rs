//! ATLAS demonstration with reactive UI and a live TITAN API request.

use patina::prelude::*;

fn contador() -> View {
    let cliques = signal(0);

    el("button")
        .class("botao")
        .on(Click, move |_| cliques.update(|n| *n += 1))
        .text("Cliques: ")
        .child(cliques)
        .build()
}

fn saudacao() -> View {
    let nome = signal(String::from("mundo"));

    el("div")
        .class("campo")
        .child(el("label").attr("for", "nome").text("Seu nome"))
        .child(
            el("input")
                .attr("id", "nome")
                .attr("value", "mundo")
                .on(Input, move |ev| {
                    nome.set(ev.value().unwrap_or_default().to_string())
                }),
        )
        .child(el("p").text_with(move || format!("Olá, {}!", nome.get())))
        .build()
}

#[cfg(target_arch = "wasm32")]
fn integracao_titan() -> View {
    use patina::http::{describe_js_error, ApiEnvelope, HttpClient, HttpResponse};
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};

    let status = signal(String::from(
        "API ainda não consultada. Inicie o TITAN e clique no botão.",
    ));
    let status_for_click = status.clone();
    let client = HttpClient::new("http://127.0.0.1:3000").with_timeout_ms(5_000);

    let button = el("button")
        .class("botao")
        .on(Click, move |_| {
            status_for_click.set("Consultando TITAN…".to_string());

            match client.get_json("/api/v1/hello") {
                Ok(promise) => {
                    let status_success = status_for_click.clone();
                    let status_error = status_for_click.clone();

                    let on_success = Closure::once_into_js(move |value: JsValue| {
                        let decoded = (|| -> Result<String, JsValue> {
                            let response = HttpResponse::from_js(&value)?;
                            let envelope = ApiEnvelope::from_js(&response.body)?;
                            if envelope.api_version != "v1" {
                                return Err(JsValue::from_str("versão de API não suportada"));
                            }
                            let message = ApiEnvelope::string_field(&envelope.data, "message")?;
                            let environment =
                                ApiEnvelope::string_field(&envelope.data, "environment")?;
                            let request_id = response.request_id.as_deref().unwrap_or("não informado");
                            Ok(format!(
                                "{message} (ambiente: {environment}; HTTP {}; request ID: {request_id})",
                                response.status
                            ))
                        })();

                        match decoded {
                            Ok(message) => status_success.set(message),
                            Err(error) => status_success.set(format!(
                                "Resposta fora do contrato esperado: {}",
                                describe_js_error(&error)
                            )),
                        }
                    });

                    let on_error = Closure::once_into_js(move |error: JsValue| {
                        status_error.set(format!(
                            "Falha ao consultar TITAN: {}",
                            describe_js_error(&error)
                        ));
                    });

                    let _ = promise.then2(on_success.unchecked_ref(), on_error.unchecked_ref());
                }
                Err(error) => status_for_click.set(format!(
                    "Não foi possível iniciar a requisição: {}",
                    describe_js_error(&error)
                )),
            }
        })
        .text("Consultar TITAN");

    el("section")
        .class("integracao")
        .child(el("h2").text("Integração com TITAN"))
        .child(el("p").text(
            "Consulta real à API versionada por HTTP/JSON, com timeout e validação do envelope.",
        ))
        .child(button)
        .child(el("p").attr("aria-live", "polite").child(status))
        .build()
}

pub fn pagina() -> View {
    let page =
        el("main")
            .class("patina-demo")
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
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    patina::mount("#app", pagina())
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
