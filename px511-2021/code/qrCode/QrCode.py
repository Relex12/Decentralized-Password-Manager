# il faut d'abord installer les libs utiles:
# pip3 install qrcode
# pip install pillow
# pip3 install opencv-python
# pip install pyautogui

import qrcode
import cv2
from tkinter import *
import pyautogui
from PIL import Image
from PIL import ImageOps
import time

def save_QrCode(msg, path):
    # Génerer un QrCode
    img = qrcode.make(msg)
    img.save(path)
    print("image", path, "sauvegardée")
    return img

# prendre un screen avec la cam
def take_screen(path, num_camera=0):
    cap = cv2.VideoCapture(num_camera)

    # Detection d'une caméra sur l'appareil
    there_is_camera = True
    while(not(cap.isOpened()) and there_is_camera):
        num_camera +=1
        cap = cv2.VideoCapture(num_camera)
        if(num_camera==10):
            print("Aucune Caméra détectée")
            there_is_camera = False
    while( cap.isOpened() ): # la cam est ouverte
        ret, frame = cap.read()
        if ret == True:
            frame = cv2.flip(frame,1)
            cv2.imshow('frame' , frame)
            # On récupère la touche appuyée par l'utilisateur
            key = cv2.waitKey(1) & 0xFF
            # On regarde laquelle c'est
            if key == ord('q'): # "q" pour sortir
                break
            if key == ord('c'): # "c" pour changer de caméra
                time.sleep(0.5) # sleep 0.5s pour ne pas sauter la caméra suivante
                take_screen(path, num_camera+1)
                break
            if key == ord('s'): # "s" pour screen (il faut un peu rester appuyer je crois, des fois ça marche pas bien)
                width  = cap.get(cv2.CAP_PROP_FRAME_WIDTH)
                height = cap.get(cv2.CAP_PROP_FRAME_HEIGHT)
                pyautogui.screenshot(path, region=(0, 0, width, height)) # prend un screen de la fenêtre décrite par region
                # pour l'instant on peut mettre à la main la fenêtre de la caméra en haut à gauche
                # mais on pourra sûrment au lieu de (0,0) dans région mettre les coordonnées de la fenêtre de la caméra
                # voir même mettre les coordonnées de la souris 
                break
        else:
            break
    cap.release()
    cv2.destroyAllWindows()

def decode_QrCode(path):
    # faire un miroir du screen pour pouvoir le décoder
    img    = Image.open(path)
    img = ImageOps.mirror(img)
    img.save("qrCode/ScreenMiror.png")

    #Décoder le screen du QrCode
    d = cv2.QRCodeDetector()
    val, points, qrCode = d.detectAndDecode(cv2.imread("qrCode/ScreenMiror.png"))
    # si le miroir ne marche pas alors on peut tester avec le screen initial
    if (val==""):
        val, points, qrCode = d.detectAndDecode(cv2.imread(path))
        # Si ça ne marche toujours pas alors on renvoie -1 signe de détection ratée
        if val == "":
            return -1
    return val.encode('UTF-8')

def read_encrypted_passwords(path):
    text = "".encode('UTF-8')
    filin = open(path, "rb")
    lines = filin.readlines()
    for line in lines:
        text +=line
    return text

def sync_other_device_QrCode():
    # On commence par récupérer les mots de passe chiffrés
    text = read_encrypted_passwords("passwords_chiffres.txt")
    # On crée et on sauvegarde le QrCode avec les mots de passe chiffrés
    img = save_QrCode(text, "qrCode/testQrCodetext.png") 
    # Puis on l'affiche pour pouvoir le scanner via un autre appareil
    img.show()

def sync_this_device_QrCode():
    # On ouvre la caméra pour prendre un screen du qrCode (on enregistre le screen à l'endroit donné en argument)
    location_screen = "qrCode/Screen.png"
    take_screen(location_screen)
    # On le décode
    result = decode_QrCode(location_screen)
    if result == -1:
        print("La détection du QrCode a échouée")
    else:
        print("detection reussie")
        # On réécrit dans le fichier passwords_chiffres avec les mots de passe mis à jours
        pws_chif_file = open("passwords_chiffres.txt", "wb")
        pws_chif_file.write(result)
        pws_chif_file.close()
        # On écrit également le résultat dans un fichier result (Pour tester et pour la démo)
        res_file = open("qrCode/result.txt", "wb")
        res_file.write(result)
        res_file.close()
        print("synchronisation terminée")
    
