# doory-reader

This contains code that reads keypad input from `doory-keypad` via usbcdc and
can also generate valid TOTP codes using a hardware token with NFC support by
way of libnfc. The combined input is sent over the network to
`doory-strikeplate`.
