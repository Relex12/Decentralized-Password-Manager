import bluetooth
import sys

# Serveur
def sync_other_device_Bluetooth():
    sock = bluetooth.BluetoothSocket(bluetooth.RFCOMM)

    print("Recherche de périphériques Bluetooth...")
    nearby_devices = bluetooth.discover_devices(lookup_names=True)

    num_bt = 1
    for addr, name in nearby_devices:
        print("addresse {0}: {1}".format(num_bt, addr))
        print("nom {0}: {1}".format(num_bt, name))
        num_bt += 1
    chosenNum = int(input("Entrer l'index du périphérique à qui envoyer les données: "))

    bt_addr = nearby_devices[chosenNum-1][0]
    port = 10 # le port 10 fonctionne bien pour la démo. Mais attention, 
            # pour une utilisation optimale, il faudra surement rechercher un port où 
            # nous sommes sûr que la conexion fonctionne.
    connected = False
    print("Tentative de connexion à {} sur le port 0x{}...".format(bt_addr, port))
    try:
        sock.connect((bt_addr, port))
        print("Connecté")
        connected = True
    except OSError as err:
        print("Une erreur est survenue lors de la phase de connexion, OSError: {0}".format(err))
    except:
        print("Une erreur est survenue lors de la phase de connexion: ", sys.exc_info()[0])

    if connected:
        with open("passwords_chiffres.txt", "rb") as myfile:
            data = myfile.read()
        a = input("Cliquer sur Entrée pour envoyer le fichier")
        try:
            sock.send(data)
            print("Fichier envoyé avec succès!")
        except OSError as err:
            print("Une erreur est survenue lors de la phase d'envoi, OSError: {0}".format(err))
        except:
            print("Une erreur est survenue lors de la phase d'envoi: ", sys.exc_info()[0])
    sock.close()



# Client
def sync_this_device_Bluetooth():
    server_sock = bluetooth.BluetoothSocket(bluetooth.RFCOMM)
    binded = False
    try:
        #print(bluetooth.PORT_ANY)
        #server_sock.bind((bluetooth.read_local_bdaddr()[0], bluetooth.PORT_ANY))
        server_sock.bind((bluetooth.read_local_bdaddr()[0], 10))
        binded = True
    except OSError as err:
        print("Une erreur est survenue lors de la phase de bind, OSError: {0}".format(err))
    except SystemError as err:
        print("Une erreur est survenue lors de la phase de bind SystemError: {0}".format(err))

    if binded:

        server_sock.listen(1)

        port = server_sock.getsockname()[1]

        uuid = "94f39d29-7d6d-437d-973b-fba39e49d4ee"

        bluetooth.advertise_service(server_sock, "SampleServer", service_id=uuid,
                                    service_classes=[uuid, bluetooth.SERIAL_PORT_CLASS],
                                    profiles=[bluetooth.SERIAL_PORT_PROFILE],
                                    # protocols=[bluetooth.OBEX_UUID]
                                    )

        print("En attente du serveur... sur le port Ox10")

        client_sock, client_info = server_sock.accept()
        print("Connexion acceptée de", client_info)

        # Erase the old content in the password file
        #file = open("passwords_chiffres.txt", "wb", errors='ignore')
        #file.close()

        try:
            while True:
                data = client_sock.recv(1024)
                if not data:
                    break
                print("Reçu", data)
                mdp_file = open("passwords_chiffres.txt", "wb")
                mdp_file.write(data)
                mdp_file.close()
        except OSError as err:
            print("Une erreur est survenue lors de la phase de réception, OSError: {0}".format(err))
        except TypeError as err:
            print("Une erreur est survenue lors de la phase de réception, TypeError: {0}".format(err))
        print("Déconnecté.")

        client_sock.close()
        server_sock.close()
        print("Fini.")