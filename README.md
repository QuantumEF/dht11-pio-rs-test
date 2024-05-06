A rust example of reading a DHT11 temperature and humidity sensor using [Embassy](https://embassy.dev/) and [PIO](https://www.raspberrypi.com/news/what-is-pio/).

The PIO assemby is adapted from the following repo: https://github.com/ashchap/PIO_DHT11_Python/blob/main/src/dht11.py

It is not completely functional, as there seems to be an issue reading the checksum byte.

It does seem to be able to read in the temperature and humidity bytes.
