provider "vault" {
  # VAULT_ADDR
  # VAULT_TOKEN
}

resource "vault_auth_backend" "approle" {
  type = "approle"
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
{"policies":"keypad"}
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
