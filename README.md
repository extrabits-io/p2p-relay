# P2P Relay

Open locally-hosted apps to the Internet.

## Use Cases

* **Developers:** allow clients to preview/demo your app without a hosting platform.
* **Businesses:** host your app privately and securely. Keep your data on your machines.
* **Hobbyists:** share local servers with friends.

## Getting Started

### Prerequisites

* Publicly-accessible server machine, such as a VPS from [Hostinger](https://www.hostinger.com/) or [Digital Ocean](https://www.digitalocean.com/).

### Public server configuration

Create a new file called `config.toml` or make a copy of `config.example.toml`.

```toml
[server]
private_key_path = "/path/to/private.key"
port_range = "3000..4000"

[[peers]]
label = "my-local-machine"
public_key = "<base-64 encoded ed25519 public key>"
```  
 __multiple peer support is coming soon__

Start the server:
```bash
$ RUST_LOG=info ./p2p-relay
```

Use a reverse proxy such as [Caddy](https://caddyserver.com/docs/quick-starts/reverse-proxy) to handle SSL certificates. Make sure it points to your local machine port, not the relay (e.g. 3001, not 3000):

```bash
caddy reverse-proxy --from my.domain.com --to localhost:3001
```

### Local machine configuration

Use [p2p-app-host](https://github.com/extrabits-io/p2p-app-host) to quickly get host a local app and connect to the relay.
