# Ordin
Ordin is a service that'll handle finialization of your VMs and containers. Ordin utilizes cloud-init's phone-home feature to do this.

## Features
- Create a DNS record for the new machine
- Run Ansible playbooks on the new machine

## Installing
Ordin can be installed using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):
```
cargo install --git https://github.com/TobiasDeBruijn/Ordin.git
```

## Usage
When you run `ordin` for the first time, a default configuration will be created at at `/etc/ordin/config.toml`, or at the path 
specified with the `-c/--config` flag. After you've configured Ordin, it will start and listen on port `4040`.

You can then configure `cloud-init` to phone home to Ordin after it is done:
```yaml
phone_home:
    url: https://ordin.example.com/phone-home
    post:
        - hostname
    tries: 10
```
Only the `hostname` post-field is required. See the [cloud-init documentation](https://cloudinit.readthedocs.io/en/latest/topics/modules.html#phone-home) for more information.

Ordin's verbosity can be controlled with the `-v/--verbose` flag, this flag can be applied multiple times.

## Configuration
By default Ordin places it's configuration into `/etc/ordin/config.toml`. This can be changed with the `-c/--config` argument.

### Example
```toml
[ansible]
# A list of Ansible playbooks to be run 
playbooks = [
    "./iptables.yaml"
]
# The ansible inventory file. New machines will be added under all.children.cloud-init.hosts
inventory = './inventory.yaml'
# Should logfiles be made for each ansible play
play_logs = false
# The logging directory for ansible plays. By default this is /var/log/ordin/
play_logdir = './logs'

[dns]
# The DNS server
# The DNS server must support DDNS
server = '127.0.0.1'
# The name of the DNS zone
zone_name = 'rpz'
# The TTL of the DNS record
ttl = 9100

[global]
# The domain to use
# E.g. if the hostname of the new machine is 'foo', and the domain is 'example.com', then it's DNS record will be set as 'foo.example.com'
domain = 'example.com'
# Should Ipv6 mode be enabled. Please note that if this is set to true, ipv4 addresses will no longer work
ipv6 = false
# The port to listen on
port = 4040
```

## Contributing
All contributions are welcome! If you discover a bug or want to add a new feature, please feel free to open an issue or a pull request. 

## License
Ordin is licensed under the [GPL-v3 license](LICENSE)
