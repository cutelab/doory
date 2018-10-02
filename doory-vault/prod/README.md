# "Production" Vault Bringup

To host the vault server we used a Raspberry Pi, running Raspbian.

The latest Raspbian image can be found [here.](https://www.raspberrypi.org/downloads/raspbian/)

After unzipping the image and using dd to copy to an SD card, the file `/boot/ssh` is created in the
boot filesystem partition of the SD card. This file allows easier access as it will bringup the ssh service
automatically on boot.

```
vault server -config=/etc/vault.hcl
```

```
vault init -key-shares=5 -key-threshold=2 -pgp-keys="alice.asc,bob.asc,carol.asc,dan.asc,erin.asc"
```

```
vault unseal
```

```
terraform plan
terraform apply
```

```
vault write auth/approle/login role_id=$(terraform output keypad_role_id)
```

```
vault write totp/keys/ev generate=true issuer=Doory account_name=doory@cutelab.house
vault write secret/prefix/1234 key=ev
```
