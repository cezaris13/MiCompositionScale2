import requests
import base64
import dotenv
import os
import jwt

from datetime import datetime

def getUserData(config):
    accessToken = config.get("ACCESS_TOKEN")
    if isAccessTokenExpired(accessToken):
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
    dotenv.set_key(os.path.dirname(__file__) + "/.env", "ACCESS_TOKEN", config.get("ACCESS_TOKEN"))
    dotenv.set_key(os.path.dirname(__file__) + "/.env", "REFRESH_TOKEN", config.get("REFRESH_TOKEN"))

def updateBodyFat(config, bodyFat, dateTime):
    accessToken = config.get("ACCESS_TOKEN")
    if isAccessTokenExpired(accessToken):
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
    if isAccessTokenExpired(accessToken):
        refreshAccessToken(config)

    header = {'Authorization' : 'Bearer {}'.format(accessToken)}
    params = {'weight': weight,
              'date' : dateTime.strftime("%Y-%m-%d"),
              'time': dateTime.strftime("%H:%M:%S")}
    response = requests.post("https://api.fitbit.com/1/user/-/body/log/weight.json", headers=header, params=params)
    return response.status_code

def isAccessTokenExpired(accessToken):
    decodedToken = jwt.decode(accessToken, options={"verify_signature": False})
    expirationTime = decodedToken['exp']
    expirationDate = datetime.fromtimestamp(expirationTime)
    return expirationDate < datetime.now()

def getAccessToken(config):
    access_code = config.get("ACCESS_CODE")
    automateTokenRetrieval(config, access_code)


def automateTokenRetrieval(config, code: str):
    data = {
        "grant_type": "authorization_code",
        "redirect_uri": "http://127.0.0.1:8080/",
        "code": code
    }
    basic_token = base64.b64encode(
        f"{config.get('CLIENT_ID')}:{config.get('CLIENT_SECRET')}".encode("utf-8")
    ).decode("utf-8")
    headers = {
        "Content-Type": "application/x-www-form-urlencoded",
        "Authorization": f"Basic {basic_token}"
    }
    response = requests.post(params=data, headers=headers,
                             url="https://api.fitbit.com/oauth2/token")
    keys = response.json()
    print(keys)
    dotenv.set_key(os.path.dirname(__file__) + "/.env", "ACCESS_TOKEN", keys["access_token"])
    dotenv.set_key(os.path.dirname(__file__) + "/.env", "REFRESH_TOKEN", keys["refresh_token"])