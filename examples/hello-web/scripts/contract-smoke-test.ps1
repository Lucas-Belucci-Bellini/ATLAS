param(
    [string]$BaseUrl = "http://127.0.0.1:3000"
)

$ErrorActionPreference = "Stop"

try {
    $baseUri = [Uri]$BaseUrl
}
catch {
    throw "BaseUrl inválida: '$BaseUrl'. Use uma URL HTTP(S), por exemplo http://127.0.0.1:3000."
}

if ($baseUri.Scheme -ne "http" -and $baseUri.Scheme -ne "https") {
    throw "BaseUrl deve usar HTTP ou HTTPS."
}

$base = $BaseUrl.TrimEnd("/")
$client = [System.Net.Http.HttpClient]::new()
$client.Timeout = [TimeSpan]::FromSeconds(10)

function Assert-Check {
    param(
        [bool]$Condition,
        [string]$Message
    )

    if (-not $Condition) {
        throw "FALHOU: $Message"
    }

    Write-Host "PASSOU: $Message" -ForegroundColor Green
}

function Invoke-SmokeGet {
    param(
        [System.Net.Http.HttpClient]$HttpClient,
        [string]$Uri
    )

    $response = $HttpClient.GetAsync($Uri).GetAwaiter().GetResult()

    try {
        $content = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $headers = @{}

        foreach ($header in $response.Headers) {
            $headers[$header.Key] = [string]::Join(", ", [string[]]$header.Value)
        }

        foreach ($header in $response.Content.Headers) {
            $headers[$header.Key] = [string]::Join(", ", [string[]]$header.Value)
        }

        return [pscustomobject]@{
            StatusCode = [int]$response.StatusCode
            Content    = $content
            Headers    = $headers
            ContentType = [string]$response.Content.Headers.ContentType.MediaType
        }
    }
    finally {
        $response.Dispose()
    }
}

try {
    Write-Host "Verificando contrato HTTP v1 em $base"

    $success = Invoke-SmokeGet -HttpClient $client -Uri "$base/api/v1/hello"
    Assert-Check ($success.StatusCode -eq 200) "GET /api/v1/hello retorna HTTP 200"
    Assert-Check ($success.ContentType -eq "application/json") "resposta de sucesso usa application/json"

    try {
        $successJson = ConvertFrom-Json -InputObject $success.Content
    }
    catch {
        throw "A resposta 200 de /api/v1/hello não contém JSON válido: $($_.Exception.Message)"
    }

    Assert-Check ($successJson.meta.api_version -eq "v1") "envelope de sucesso identifica api_version=v1"
    Assert-Check (-not [string]::IsNullOrWhiteSpace([string]$successJson.data.message)) "envelope de sucesso contém data.message"
    Assert-Check (-not [string]::IsNullOrWhiteSpace([string]$successJson.data.environment)) "envelope de sucesso contém data.environment"

    $notFound = Invoke-SmokeGet -HttpClient $client -Uri "$base/api/v1/not-found"
    Assert-Check ($notFound.StatusCode -eq 404) "GET /api/v1/not-found retorna HTTP 404"
    Assert-Check ($notFound.ContentType -eq "application/json") "resposta de erro usa application/json"

    try {
        $errorJson = ConvertFrom-Json -InputObject $notFound.Content
    }
    catch {
        throw "A resposta 404 de /api/v1/not-found não contém JSON válido: $($_.Exception.Message)"
    }

    Assert-Check ($errorJson.meta.api_version -eq "v1") "envelope de erro identifica api_version=v1"
    Assert-Check (-not [string]::IsNullOrWhiteSpace([string]$errorJson.error.code)) "envelope de erro contém error.code"
    Assert-Check (-not [string]::IsNullOrWhiteSpace([string]$errorJson.error.message)) "envelope de erro contém error.message"
    Assert-Check (-not [string]::IsNullOrWhiteSpace([string]$notFound.Headers["x-request-id"])) "resposta de erro contém x-request-id"

    Write-Host ""
    Write-Host "Contrato HTTP v1 verificado com sucesso." -ForegroundColor Green
}
finally {
    $client.Dispose()
}
