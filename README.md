# cosmoping

Latency report generator for Cosmos SDK addr book

## Usage

To get a basic latency report run cosmoping against a CosmosSDK addrbook.json file:

```bash
cosmoping latency --addrbook-path <path>
```

The terminal will print the results in markdown format:

```markdown
| IP Address | Port   | ID         | Latency(ms) | City | Country |
| ---------- | ------ | ---------- | ----------- | ---- | ------- |
| <IP1>      | <Port> | <Peer Id1> | 36354       |      |         |
| <IP2>      | <Port> | <Peer Id2> | 36328       |      |         |
```

### Results output

You can supply an output path to save the results to file:

```bash
cosmoping latency --addrbook-path <path> --output-path ./latencies.md
```

### Location data

You can get additional IP location data by supplying an api key from [IpInfo](https://ipinfo.io/):

```bash
cosmoping latency --addrbook-path <path> --output-path ./latencies.md --location-api-key <API KEY>
```

This will enrich the resolvable IPs with locations:

```markdown
| IP Address | Port   | ID         | Latency(ms) | City    | Country    |
| ---------- | ------ | ---------- | ----------- | ------- | ---------- |
| <IP1>      | <Port> | <Peer Id1> | 36354       | <City1> | <Country1> |
| <IP2>      | <Port> | <Peer Id2> | 36328       | <City2> | <Country1> |
```
