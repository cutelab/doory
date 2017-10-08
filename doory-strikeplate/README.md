# doory-strikeplate

This receives pin / TOTP code pairs for every entry attempt over
the network from `doory-reader`. It queries a Vault API endpoint
to determine validity. (The Vault socket is forwarded via SSH to be available on the local host.)
When a valid pin / TOTP code pair is received, a GPIO is toggled
which is connected to a relay controlling the door strikeplate.
