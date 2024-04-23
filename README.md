# flakysaas

```
docker build -t flakysaas .
docker run -d --restart unless-stopped -p 9001:9001 flakysaas
```

## Release

On prod machine, run `./deploy`.


## API

```
curl http://localhost:9001/currencies
curl -H 'Content-Type: application/json' -d '{"quote": "USD", "base": "BTC"}' http://localhost:9001/quote
```
