import logging

log = logging.getLogger()


def basicConfig(**kwargs):
    kwargs["format"] = "%(asctime)s - [%(levelname)s] - %(name)s - (%(filename)s:%(lineno)d).%(funcName)s - %(message)s"
    logging.basicConfig(**kwargs)
