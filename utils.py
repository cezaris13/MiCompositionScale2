from pytz import timezone

def unitToKg(value, unit):
    if unit == 'jin':
        return value / 1.66667
    if unit == 'lbs':
        return value / 2.205
    return value

def datetimeToTimezone(dateTime, timezoneString):
    converted_tz = timezone(timezoneString)
    print(converted_tz)


    return dateTime.astimezone(timezone(timezoneString))