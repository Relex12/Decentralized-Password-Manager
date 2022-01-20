from QrCode import *

# Sert pour tester sur un même pc à la fois la génération du QrCode et la lecture de celui ci
if __name__ == '__main__':
    sync_other_device_QrCode()
    sync_this_device_QrCode()