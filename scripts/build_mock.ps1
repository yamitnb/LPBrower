$env:VITE_MOCK = "1"
Write-Host "VITE_MOCK is: $env:VITE_MOCK"
npx vite build --outDir dist-mock
