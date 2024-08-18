Get Xiaomi Mi Composition Scale 2 weight and publishing to fitbit

## Requirements

 * python3
 * python-dotenv
 * bluepy
 * root permission for `bluepy.btle`

```bash
sudo make install
```

## Usage

always run with `sudo` or from `root`:

```bash
cp .env.dist .env
vim .env
sudo make run
# sudo ./main.py --help
# sudo make debug
```

## Autostart

```bash
sudo make add-service
sudo make enable-service

```
## Help

get dev mac address:

```bash
sudo hcitool lescan
```

if u have troubleshoots, try restart u bluetooth/adapter

```bash
sudo hciconfig hci0 reset
sudo invoke-rc.d bluetooth restart
```

### Reverse Engineering RAW Schema for Mi Composition Scale 2

!!! *slightly different than from openScale wiki* !!!

**byte 0:**

- 0 bit - unknown
- 1 bit - unit kg
- 2 bit - unit lbs
- 3 bit - unknown
- 4 bit - jin (chinese catty) unit
- 5 bit - stabilized
- 6 bit - unknown
- 7 bit - weight removed

**byte 1-2:**
 - weight (little endian)

## Links

 * https://github.com/oliexdev/openScale/wiki/Xiaomi-Bluetooth-Mi-Scale
