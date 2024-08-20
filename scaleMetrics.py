from datetime import *

def processPacket(rawData):
    isLbs = bool(rawData[0] & 1)
    hasImpedance = bool(rawData[1] & (1 << 1))
    isStabilized = bool(rawData[1] & (1 << 5))
    isJin = bool(rawData[1] & (1 << 6) )
    isWeightRemoved = bool(rawData[1] & (1 << 7))
    weight = int.from_bytes(rawData[11:13], byteorder="little") / 100
    impedance = int.from_bytes(rawData[9:11], byteorder="little")
    year = int.from_bytes(rawData[2:4], byteorder="little")
    month = rawData[4]
    day = rawData[5]
    hour = rawData[6] # time is in GMT+0
    minutes = rawData[7]
    seconds = rawData[8]
    dateTime = datetime(year, month, day, hour, minutes, seconds)
    if isJin:
        unit = "jin"
    elif isLbs:
        unit = "lbs"
    else:
        unit = "kg"
        weight /= 2  # lbs to kg

    return weight, unit, hasImpedance, impedance, isStabilized, isWeightRemoved, dateTime

def getFatPercentage(impedance, weight, sex, age, height):
    sex = sex.lower()
    # Set a constant to remove from LBM
    if sex == 'female' and age <= 49:
        const = 9.25
    elif sex == 'female' and age > 49:
        const = 7.25
    else:
        const = 0.8

    # Calculate body fat percentage
    LBM = getLBMCoefficient(height, weight, impedance, age)

    if sex == 'male' and weight < 61:
        coefficient = 0.98
    elif sex == 'female' and weight > 60:
        coefficient = 0.96
        if height > 160:
            coefficient *= 1.03
    elif sex == 'female' and weight < 50:
        coefficient = 1.02
        if height > 160:
            coefficient *= 1.03
    else:
        coefficient = 1.0
    fatPercentage = (1.0 - (((LBM - const) * coefficient) / weight)) * 100

    # Capping body fat percentage
    if fatPercentage > 63:
        fatPercentage = 75
    return checkValueOverflow(fatPercentage, 5, 75)


# Get LBM coefficient (with impedance)
def getLBMCoefficient(height, weight, impedance, age):
    lbm =  (height * 9.058 / 100) * (height / 100)
    lbm += weight * 0.32 + 12.226
    lbm -= impedance * 0.0068
    lbm -= age * 0.0542
    return lbm

def checkValueOverflow(value, minimum, maximum):
    if value < minimum:
        return minimum
    elif value > maximum:
        return maximum
    else:
        return value