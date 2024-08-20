import requests
import base64
import json

def getUserData(config):
    accessToken = config.get("ACCESS_TOKEN")
    if isAccessCodeExpired(accessToken):
        refreshAccessToken(config)

    header = {'Authorization' : 'Bearer {}'.format(accessToken)}
    response = requests.get("https://api.fitbit.com/1/user/-/profile.json", headers=header)
    responseData = response.json()
    user = responseData['user']
    age = user['age']
    gender = user['gender']
    height = user['height']
    weight = user['weight']
    timeZone = user['timezone']
    return gender, age, height, weight, timeZone

def refreshAccessToken(config):
    refreshToken = config.get("REFRESH_TOKEN")
    clientId = config.get("CLIENT_ID")
    clientSecret = config.get("CLIENT_SECRET")
    encodedClientData = base64.b64encode((clientId+":"+clientSecret).encode())
    header = {'Authorization' : 'Basic {}'.format(encodedClientData.decode()),
              'Content-Type':'application/x-www-form-urlencoded'}
    params = {'refresh_token': '{}'.format(refreshToken),
              'grant_type' : 'refresh_token'}
    response = requests.post("https://api.fitbit.com/oauth2/token", headers=header, params=params)

    print(response.status_code)
    print(response.json())
    accessToken = response.json()['access_token']
    refreshToken = response.json()['refresh_token']
    config.__setattr__("ACCESS_TOKEN", accessToken)
    config.__setattr__("REFRESH_TOKEN", refreshToken)

def updateBodyFat(config, bodyFat, dateTime):
    accessToken = config.get("ACCESS_TOKEN")
    if isAccessCodeExpired(accessToken):
        refreshAccessToken(config)

    header = {'Authorization' : 'Bearer {}'.format(accessToken)}
    params = {'fat': bodyFat,
              'date' : dateTime.strftime("%Y-%m-%d"),
              'time': dateTime.strftime("%H:%M:%S")}
    print(params)
    response = requests.post("https://api.fitbit.com/1/user/-/body/log/fat.json", headers=header, params=params)
    return response.status_code

def updateBodyWeight(config, weight, dateTime):
    accessToken = config.get("ACCESS_TOKEN")
    if isAccessCodeExpired(accessToken):
        refreshAccessToken(config)

    header = {'Authorization' : 'Bearer {}'.format(accessToken)}
    params = {'weight': weight,
              'date' : dateTime.strftime("%Y-%m-%d"),
              'time': dateTime.strftime("%H:%M:%S")}
    print(params)
    response = requests.post("https://api.fitbit.com/1/user/-/body/log/weight.json", headers=header, params=params)
    return response.status_code


def isAccessCodeExpired(accessCode): # work on this
    return False