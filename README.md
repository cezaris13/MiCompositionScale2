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

Payload format (year, impedance and weight are little endian):

* bytes 0 and 1: control bytes
* bytes 2 and 3: year
* byte 4: month
* byte 5: day
* byte 6: hours
* byte 7: minutes
* byte 8: seconds
* bytes 9 and 10: impedance
* bytes 11 and 12: weight (`/100` for pounds and catty, `/200` for kilograms)

Control bytes format (LSB first):

* bit 0:   unknown
* bit 1:   unknown
* bit 2:   unknown
* bit 3:   unknown
* bit 4:   unknown
* bit 5:   unknown
* bit 6:   unknown (always 1 on my scale)
* bit 7:   is pounds
* bit 8:   is empty load (no weight on scale)
* bit 9:   is catty
* bit 10:  is stabilized (weight confirmed, that's also when the weight on scale blinks)
* bit 11:  unknown
* bit 12:  unknown
* bit 13:  unknown (always 1 on my scale)
* bit 14:  have impedance (impedance bytes are set correctly)
* bit 15:  unknown