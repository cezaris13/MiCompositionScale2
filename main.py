#!/usr/bin/env python3

import argparse
import logging
import os

from dotenv import dotenv_values

from logger import log, basicConfig
from fitbitData import getLastWeight, updateBodyFat, updateBodyWeight
from scaleMetrics import getFatPercentage
from scanner import start


def main():
    config = dotenv_values(os.path.dirname(__file__) + "/.env")
    parser = argparse.ArgumentParser()
    parser.add_argument("--loglevel", dest="logLevel", choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"],
                        help="set the logging level")

    args = parser.parse_args()
    if args.logLevel:
        basicConfig(level=getattr(logging, args.logLevel))

    def callback(weight, unit, hasImpedance, impedance, dateTime):
        log.info("received data = %s %s", weight, unit)
        lastWeight = getLastWeight()

        # https://www.healthline.com/health/weight-fluctuation
        # If someone finds more reliable source, create an issue.
        if lastWeight - 3 < weight < lastWeight + 3:
            if hasImpedance:
                updateBodyFat(getFatPercentage(impedance, weight), dateTime)
            updateBodyWeight(weight, dateTime)
        else:
            log.warning("weight is not between %s and %s, skip publishing", config.get("MIN_WEIGHT"),
                        config.get("MAX_WEIGHT"))

    start(config.get("MAC_ADDRESS"), float(config.get("TIMEOUT")), callback)


if __name__ == "__main__":
    main()
