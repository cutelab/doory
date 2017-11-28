# doory-vault 

This contains notes and configuration for bringing up a Hashicorp Vault
instance for entry control.

## general info

The doory vault uses two secret backends:
* [generic key/value](https://www.vaultproject.io/api/secret/kv/index.html)
* [totp](https://www.vaultproject.io/api/secret/totp/index.html)

the generic key/value secret backend is used to lookup the totp path based on the pin
prefix. the totp secret backend is used to validate totp codes.

The doory vault uses auth backends:
* [token](https://www.vaultproject.io/api/auth/token/index.html)
* [approle](https://www.vaultproject.io/api/auth/approle/index.html)

These auth backends are used by the `door-strikeplate` subproject,
which has scoped permissions to allow reading the key/vault secret
backend and check codes against the totp backend.

Token auth can be used directly for dev, but we will likely want to
use approle for production. Currently fetching a token using approle
creds is not implemented.

The terraform files in `./tf` handle mounting secret and auth backends
described above, setting up the scoped permissions and creating the
keypad role for token authentication. 

Other auth backends should be used in production by admins to provision new
TOTP credentials, static codes and prefixes. (e.g. Username & Password)

For dev use the root credential is used to provision credentials etc.

## bringing up a dev vault
```
docker-compose build
docker-compose up -d
```

## setting local env for dev vault
```
export VAULT_ADDR=http://localhost:8200
export VAULT_TOKEN=$(docker logs vault 2>&1 | grep "Root Token" | cut -d ' ' -f 3 )
```

## mounting secret backends and provisioning vault
```
cd tf
terraform apply
```

## creating dev token for use by doory-strikeplate
```
vault token-create -policy keypad -orphan
# the response "token" value can be used by doory-strikeplate by setting
# VAULT_TOKEN in it's environment
```

## user totp secret / prefix creation
```
vault write totp/keys/ev generate=true account_name=ev issuer=cutelabs qr_size=0
vault write secret/prefix/2626 key=ev
```
