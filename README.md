# Dynamic DNS updater

A simple project to update the Dynu dynamic DNS when the local IP address changes.

The project is split into 2 components:

- **server:** returns the IP address of the client
- **client:** calls the _server_ to obtain the public IP address, compares it agains the current address (querying the DNS server) and updates Dynu if they're different

Server example:

```sh
./dynamicdns-rupdater --server --host 0.0.0.0 --port 8081
```

Client example:

```sh
./dynamicdns-rupdater \
    --client \
    --host <server address> \
    --hostname <dns record> \
    --username <user> \
    --password <psw>
```
