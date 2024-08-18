program-name = composition-scale-2
main-file = main.py

install:
	apt-get install python3-pip libglib2.0-dev bluez
	python -m pip install -r requirements.txt --break-system-packages

run:
	python $(main-file)

debug:
	python $(main-file) --loglevel DEBUG

add-service:
	cp $(program-name).service /etc/systemd/system/

enable-service:
	systemctl enable $(program-name)
	systemctl start $(program-name)

disable-service:
	systemctl disable $(program-name)