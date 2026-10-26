# Simple script to set up .env file with your GROQ API key

Write-Host "IntelliShell Environment Setup" -ForegroundColor Cyan
Write-Host ""
Write-Host "You need a GROQ API key to use AI features." -ForegroundColor Yellow
Write-Host "Get one free at: https://console.groq.com/keys" -ForegroundColor Yellow
Write-Host ""

$groqKey = Read-Host "Enter your GROQ API key (starts with gsk_)"

if ($groqKey -match "^gsk_") {
    # Update .env file
    $envContent = Get-Content ".env"
    $envContent = $envContent -replace "GROQ_API_KEY=.*", "GROQ_API_KEY=$groqKey"
    $envContent | Set-Content ".env"
    
    Write-Host ""
    Write-Host "GROQ API key saved to .env" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next step: Run docker-run.ps1 to start IntelliShell" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "Invalid key format. GROQ keys start with gsk_" -ForegroundColor Red
    Write-Host "Try again or edit .env manually" -ForegroundColor Yellow
}
