from pytz import timezone
from datetime import timedelta

def unitToKg(value, unit):
    if unit == 'jin':
        return value / 1.66667
    if unit == 'lbs':
        return value / 2.205
    return value

def datetimeToTimezone(dateTime, timezoneString):
    convertedTimezone = timezone(timezoneString)
    local_time = dateTime.astimezone(convertedTimezone)
    timeDifference = local_time.utcoffset().total_seconds() / 3600
    return dateTime + timedelta(hours=timeDifference)