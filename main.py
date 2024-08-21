#!/usr/bin/env python3

import argparse
import logging
import os

from dotenv import dotenv_values

from logger import log, basicConfig
from fitbitData import updateBodyFat, updateBodyWeight, getUserData, getAccessToken
from scaleMetrics import getFatPercentage
from scanner import start
from utils import unitToKg, datetimeToTimezone

def main():
    config = dotenv_values(os.path.dirname(__file__) + "/.env")
    parser = argparse.ArgumentParser()
    parser.add_argument("--loglevel", dest="logLevel", choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"],
                        help="set the logging level")

    args = parser.parse_args()
    if args.logLevel:
        basicConfig(level=getattr(logging, args.logLevel))

    if config.get("ACCESS_TOKEN") == "" or config.get("REFRESH_TOKEN") == "":
        getAccessToken(config)

    def callback(weight, unit, hasImpedance, impedance, datetime):
        log.info("received data = %s %s, %s %s", weight, unit, hasImpedance, impedance)
        sex, age, height, lastWeight, timezone = getUserData(config) # seems that weight and height data is saved in metric so no conversion needed
        weight = unitToKg(weight, unit)
        datetime = datetimeToTimezone(datetime, timezone)

        # https://www.healthline.com/health/weight-fluctuation
        # If someone finds more reliable source, create an issue.
        if lastWeight - 3 < weight < lastWeight + 3:
            if hasImpedance:
                updateBodyFat(config, getFatPercentage(impedance, weight, sex, age, height), datetime)
            updateBodyWeight(config, weight, datetime)
        else:
            log.warning("weight is not between %s and %s, skip publishing", config.get("MIN_WEIGHT"),
                        config.get("MAX_WEIGHT"))

    start(config.get("MAC_ADDRESS"), float(config.get("TIMEOUT")), callback)

if __name__ == "__main__":
    main()
