# doory (A TOTP Keypad for Dore)

## Overview
```
10.0.40.x                                   10.0.40.y | 192.168.1.1       192.168.1.100
 ( vault ) -> foward vault http api over ssh -> ( strikeplate ) <- udp <- ( nfc reader ) <- usb <- ( keypad )
                                                                <= ntp =>
```

## TODO
* add vault notes and bringup code
* ntp server on strikeplate for use by reader
* add static code support
* make small wrapper cli for adding users?
