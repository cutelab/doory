provider "vault" {
  # VAULT_ADDR
  # VAULT_TOKEN
}

resource "vault_auth_backend" "approle" {
  type = "approle"
}

resource "vault_generic_secret" "approle_tune" {
  depends_on = ["vault_auth_backend.approle"]
  path = "/sys/mounts/auth/approle/tune"
  data_json = <<EOT
  { "max_lease_ttl": "31536000" }
EOT
}

resource "vault_mount" "totp" {
  path = "totp"
  type = "totp"
}

resource "vault_policy" "keypad" {
  name = "keypad"

  policy = <<EOT
path "*" {
  capabilities = ["deny"]
}

path "secret/static/*" {
  capabilities = ["list", "read"]
}

path "secret/prefix/*" {
  capabilities = ["list", "read"]
}

path "totp/code/*" {
  capabilities = ["update"] # ( Validate: POST	/totp/code/:name )
}
EOT
}

resource "vault_generic_secret" "approle_keypad_role" {
  depends_on = ["vault_auth_backend.approle"]
  path = "auth/approle/role/keypad"

  data_json = <<EOT
{"policies":"keypad", "token_ttl":"8760h", "token_max_ttl": "8760h", "bind_secret_id": "false", "secret_id_bound_cidrs":["127.0.0.1/32"]}
EOT
}

data "vault_generic_secret" "keypad_role_id" {
  depends_on = ["vault_generic_secret.approle_keypad_role"]
  path = "auth/approle/role/keypad/role-id"
}

output "keypad_role_id" {
  value = "${data.vault_generic_secret.keypad_role_id.data["role_id"]}"
}

resource "vault_generic_secret" "token_keypad_role" {
  path = "auth/token/roles/keypad"

  data_json = <<EOT
{"policies":"keypad"}
EOT
}
