# ATLAS — integração de demonstração com TITAN

Este exemplo mostra a UI reativa do ATLAS e, no navegador/WASM, um botão que consulta o endpoint HTTP v1 do TITAN.

## Estado desta demonstração

- O cliente HTTP usa o Fetch API do navegador.
- O cliente aplica timeout por requisição, rejeita status HTTP fora de 2xx e rejeita respostas que não possam ser interpretadas como JSON.
- A demonstração valida a versão do envelope e os campos de texto retornados.
- A chamada de rede só é incluída na compilação WASM; o conjunto de testes nativos não a executa.
- A integração real depende do TITAN estar executando e CORS permitir a origem exata da página.

## Iniciar TITAN (PowerShell)

No clone do repositório TITAN:

```powershell
$env:FERROVIA_CORS_ORIGINS = "http://localhost:8000"
cargo run -p ferrovia-server
```

O exemplo de TITAN escuta por padrão em `127.0.0.1:3000`. Acesse primeiro `http://127.0.0.1:3000/api/v1/hello` para inspecionar o JSON diretamente.

## Construir e servir ATLAS (WASM)

Na raiz do repositório ATLAS:

```powershell
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build examples/hello-web --target web
cd examples/hello-web
python -m http.server 8000
```

Abra `http://localhost:8000` e clique em **Consultar TITAN**.

Importante: a origem precisa corresponder exatamente à configuração CORS do TITAN. Se servir em outro host/porta, ajuste `FERROVIA_CORS_ORIGINS` para essa origem e reinicie o servidor.

## Limitações deliberadas

- Não envia cookies nem credenciais automaticamente.
- Não implementa autenticação/autorização.
- A resposta é um envelope JS verificado em runtime; não existe ainda geração de tipos compartilhados entre os repositórios.
- O cliente não está testado em browser automatizado nesta etapa. Faça o build WASM e uma verificação manual antes de considerar o exemplo validado.
