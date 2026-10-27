# ATLAS — integração de demonstração com TITAN

Este exemplo mostra a UI reativa do ATLAS e, no navegador/WASM, botões para consultar o endpoint de sucesso e exercitar o contrato de erro v1 do TITAN.

## Estado desta demonstração

- O cliente HTTP usa o Fetch API do navegador.
- O cliente aplica timeout por requisição, rejeita status HTTP fora de 2xx e rejeita respostas que não possam ser interpretadas como JSON.
- A demonstração valida a versão do envelope de sucesso e os campos de texto retornados.
- O botão **Testar erro v1** chama intencionalmente `/api/v1/not-found`; a mensagem apresentada deve incluir `HTTP 404`, `not_found` e o request ID quando disponível.
- `fixtures/http-v1-error.json` documenta a forma esperada para um erro versionado.
- A chamada de rede só é incluída na compilação WASM; o conjunto de testes nativos não a executa.
- A integração real depende do TITAN estar executando e CORS permitir a origem exata da página.

## Iniciar TITAN (PowerShell)

No clone do repositório TITAN:

```powershell
$env:FERROVIA_CORS_ORIGINS = "http://localhost:8000"
cargo run -p ferrovia-server
```

O exemplo de TITAN escuta por padrão em `127.0.0.1:3000`. Acesse primeiro `http://127.0.0.1:3000/api/v1/hello` para inspecionar o JSON diretamente. Para testar o erro, o exemplo do ATLAS chama `/api/v1/not-found`, que deve responder com o envelope de erro v1.

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


## Verificar o contrato HTTP sem navegador (PowerShell)

Com o TITAN em execução em outro terminal, na raiz do clone do ATLAS execute:

```powershell
.\examples\hello-web\scripts\contract-smoke-test.ps1 -BaseUrl "http://127.0.0.1:3000"
```

O script verifica o HTTP 200 e o envelope de sucesso de `/api/v1/hello`, depois verifica o HTTP 404, o envelope de erro v1 e o cabeçalho `x-request-id` de `/api/v1/not-found`. Ele usa o cliente HTTP do .NET e não exige instalar módulos PowerShell adicionais.

Este é um teste de integração contra um servidor em execução; ele não substitui o teste de interação no navegador/WASM e não testa CORS, pois a chamada é feita diretamente pelo script.

## Limitações deliberadas

- Não envia cookies nem credenciais automaticamente.
- Não implementa autenticação/autorização.
- A resposta é um envelope JS verificado em runtime; não existe ainda geração de tipos compartilhados entre os repositórios.
- O cliente não está testado em browser automatizado nesta etapa. Faça o build WASM e uma verificação manual antes de considerar o exemplo validado.
