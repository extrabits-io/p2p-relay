# P2P Relay

Open locally-hosted apps to the Internet.

## Use Cases

* **Developers:** allow clients to preview/demo your app without a hosting platform.
* **Businesses:** host your app privately and securely. Keep your data on your machines.
* **Hobbyists:** share local servers with friends.

## Getting Started

### Prerequisites

* Publicly-accessible server machine, such as a VPS from [Hostinger](https://www.hostinger.com/) or [Digital Ocean](https://www.digitalocean.com/).

* **[Wireguard](https://www.wireguard.com/)** - must be installed on the public and local machines. Any pre-existing Wireguard configuration on the public machine will be ignored.

### Public server configuration

Create a new file called `config.toml` or make a copy of `config.example.toml`.

```toml
[proxy]
listen_url = "localhost"
listen_port = 5000

[server]
ip_range = "10.1.0.1/31"

[[peers]]
label = "my-local-machine"
public_key = "<base-64 encoded Wireguard public key>"
port = 3000
```

Start the server:
```bash
$ RUST_LOG=info ./p2p-relay
```

Use a reverse proxy such as [Caddy](https://caddyserver.com/docs/quick-starts/reverse-proxy) to handle SSL certificates:

```bash
caddy reverse-proxy --from my.domain.com --to localhost:5000
```

### Local machine configuration

Follow the Wireguard quickstart [instructions](https://www.wireguard.com/quickstart/).

Server your local app on the Wireguard network:

```bash
$ HOST=10.1.0.2 PORT=3000 bun run dev
```
