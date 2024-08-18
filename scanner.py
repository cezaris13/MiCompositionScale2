from bluepy.btle import Scanner, DefaultDelegate

from logger import log


class ScanDelegate(DefaultDelegate):
    SERVICE_DATA = 22  # [1b18828809e4070310112302]

    def __init__(self, mac_address, callback):
        DefaultDelegate.__init__(self)
        self.mac_address = mac_address.upper()
        self.last_raw_data = None
        self.callback = callback

    def handleDiscovery(self, dev, isNewDev, isNewData):
        if self.mac_address == dev.addr.upper():
            self.parse_data(dev)

    def parse_data(self, dev):
        log.debug("device %s is %s, rssi: %d dBm, connectable: %s",
                  dev.addr, dev.addrType, dev.rssi, dev.connectable)
        for (tag, _, value) in dev.getScanData():
            if tag == self.SERVICE_DATA and value.startswith("1b18"): # body composition
                log.info(value)
                raw_data = bytes.fromhex(value[4:])
                if raw_data == self.last_raw_data:
                    log.debug("skip duplicate data")
                    return

                self.last_raw_data = raw_data
                ctrlBit = raw_data[1]
                is_kg = (ctrlBit & 1 << 1) != 0 # work on this
                is_lbs = (ctrlBit & (1 << 2)) != 0 # and this
                is_jin = (ctrlBit & ( 1 << 4) ) != 0 # and this
                is_stabilized = (ctrlBit & (1 << 5)) != 0 # this works
                is_weight_removed = (ctrlBit & (1 << 7)) != 0 # this works
                print("is lbs", is_lbs)
                print("is kg", is_kg)
                print("is jin", is_jin)
                print("is stabilized", is_stabilized)
                print("is weight removed", is_weight_removed)

                weight = (((raw_data[12] & 0xFF) << 8) | (raw_data[11] & 0xFF)) / 200 # this is done

                if is_jin:
                    unit = "jin"
                elif is_lbs:
                    unit = "lbs"
                elif is_kg:
                    unit = "kg"
                    weight /= 2  # catty to kg
                else:
                    unit = "unknown"

                print("weight:", weight, unit)
                if is_stabilized is True and is_weight_removed is False:
                    self.callback(weight, unit)


def start(mac_address, timeout, callback):
    log.info("scanner is starting...")
    scanner = Scanner().withDelegate(ScanDelegate(mac_address, callback))

    while True:
        scanner.start()
        scanner.process(timeout)
        scanner.stop()
