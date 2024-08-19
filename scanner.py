from bluepy.btle import Scanner, DefaultDelegate

from logger import log
from scaleMetrics import getFatPercentage, processPacket


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
                rawData = bytes.fromhex(value[4:])
                if rawData == self.last_raw_data:
                    log.debug("skip duplicate data")
                    return

                self.last_raw_data = rawData
                weight, unit, hasImpedance, impedance, isStabilized, isWeightRemoved, dateTime = processPacket(rawData)
                if isStabilized is True and isWeightRemoved is False:
                    print("weight:", weight, unit)
                    self.callback(weight, unit)


def start(mac_address, timeout, callback):
    log.info("scanner is starting...")
    scanner = Scanner().withDelegate(ScanDelegate(mac_address, callback))

    while True:
        scanner.start()
        scanner.process(timeout)
        scanner.stop()
