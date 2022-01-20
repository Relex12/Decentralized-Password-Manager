from tkinter import *
from enum import Enum
from qrCode.QrCode import *
from bluetoothFolder.RunBluetooth import *
from ice.ICE import *
from Crypto import Cipher
from Crypto.Cipher import AES
from Crypto.Util import Padding
import hashlib


class Actions(Enum):
    NONE = 0
    SYNC_THIS = 1
    SYNC_OTHER = 2

class Methods(Enum):
    NONE = 0
    QRCODE = 1
    BLUETOOTH = 2
    ICE = 3

method = Methods.NONE
action = Actions.NONE


def fenetre_action():
    #Fenetre
    fenetre = Tk()
    fenetre.title("Partage des mots de passe")
    fenetre.geometry("1080x720")
    fenetre.config(background='#028fdb')

    #Texte introduction
    intro=Label(fenetre, text="Veuillez choisir si vous voulez partager ou reçevoir.",font=("Courrier", 20),bg='#028fdb',fg="white")
    intro.pack(expand=YES)

    #Bouton "Recevoir"
    frameRE= Frame(fenetre, bg='#028fdb')
    boutonRE= Button(frameRE, text="Recevoir",font=("Courrier", 20),bg='white',fg="#028fdb",command= lambda:Sync_this(fenetre))
    boutonRE.pack()

    #Bouton "Envoyer"
    frameEN= Frame(fenetre, bg='#028fdb')
    boutonEN= Button(frameEN, text="Envoyer",font=("Courrier", 20),bg='white',fg="#028fdb",command= lambda:Sync_other(fenetre))
    boutonEN.pack()

    frameEN.pack(expand=YES)
    frameRE.pack(expand=YES)

    fenetre.mainloop()

def fenetre_method():
    #On crée la deuxième
    fenetre2=Tk()
    fenetre2.title("Partage des mots de passe")
    fenetre2.geometry("1080x720")
    fenetre2.config(background='#028fdb')

    #Texte introduction
    intro=Label(fenetre2, text="Veuillez choisir un mode de partage de vos mots de passe.",font=("Courrier", 20),bg='#028fdb',fg="white")
    intro.pack()

    #QRCode
    frameQR= Frame(fenetre2, bg='#028fdb')
    boutonQR= Button(frameQR, text="QR Code",font=("Courrier", 20),bg='white',fg="#028fdb", command=lambda:QRCode(fenetre2))
    boutonQR.pack()

    #Bluetooth
    frameBT= Frame(fenetre2, bg='#028fdb')
    boutonBT= Button(frameBT, text="Bluetooth",font=("Courrier", 20),bg='white',fg="#028fdb", command=lambda:Bluetooth(fenetre2))
    boutonBT.pack()

    #ICE
    frameICE= Frame(fenetre2, bg='#028fdb')
    boutonICE= Button(frameICE, text="ICE",font=("Courrier", 20),bg='white',fg="#028fdb", command=lambda:ICE(fenetre2))
    boutonICE.pack()

    frameQR.pack(expand=YES)
    frameBT.pack(expand=YES)
    frameICE.pack(expand=YES)

    fenetre2.mainloop()

def Sync_this(fenetre):
    global action
    action = Actions.SYNC_THIS
    #On enlève la première fenêtre
    fenetre.destroy()
    #On crée la deuxième
    fenetre_method()
    

def Sync_other(fenetre):
    #on chiffre avant d'envoyer
    chiffrement_fichier('passwords.txt','passwords_chiffres.txt',masterPassword,b"0123456789abcdef")
    global action
    action = Actions.SYNC_OTHER
    #On enlève la première fenêtre
    fenetre.destroy()
    #On crée la deuxième
    fenetre_method()

def QRCode(fenetre2):
    print("Méthode QRCode sélectionnée")
    global method
    method = Methods.QRCODE
    fenetre2.destroy()

    
def Bluetooth(fenetre2):
    print("Méthode Bluetooth sélectionnée")
    global method
    method = Methods.BLUETOOTH
    fenetre2.destroy()
    
def ICE(fenetre2):
    print("Méthode ICE sélectionnée")
    global method
    method = Methods.ICE
    fenetre2.destroy()

def sync_this_device(method):
    if method == Methods.QRCODE:
        print("On lance la fonction \"sync_this_device_QrCode\" ")
        sync_this_device_QrCode()
        dechiffrement_fichier('passwords_chiffres.txt','passwords.txt',masterPassword,b"0123456789abcdef")
    elif method == Methods.BLUETOOTH:
        print("On lance la fonction \"sync_this_device_Bluetooth\" ")
        sync_this_device_Bluetooth()
        dechiffrement_fichier('passwords_chiffres.txt','passwords.txt',masterPassword,b"0123456789abcdef")
    elif method == Methods.ICE:
        print("On lance la fonction \"sync_this_device_Ice\" ")
        sync_this_device_Ice()
        dechiffrement_fichier('passwords_chiffres.txt','passwords.txt',masterPassword,b"0123456789abcdef")
    elif method == Methods.NONE:
        print("You have to choose a method of syncronisation")

def sync_other_device(method):
    if method == Methods.QRCODE:
        print("On lance la fonction \"sync_other_device_QrCode\" ")
        sync_other_device_QrCode()
    elif method == Methods.BLUETOOTH:
        print("On lance la fonction \"sync_other_device_Bluetooth\" ")
        sync_other_device_Bluetooth()
    elif method == Methods.ICE:
        print("On lance la fonction \"sync_other_device_Ice\" ")
        sync_other_device_Ice()
    elif method == Methods.NONE:
        print("You have to choose a method of syncronisation")


def readMasterPassword():
    filin = open("master_password.txt", "r")
    mp = filin.readline()
    n = 0
    for c in mp:
        if c == '\r' or c == '\n' or c == '\t' or c == '\s':
            return(mp[:n])
        else:
            n+=1
    return mp

def chiffrement_mp(mdp):
    digest=mdp.hexdigest()
    for i in range(99999):
        tmp = hashlib.sha256()
        tmp.update(digest.encode('UTF-8'))
        digest=tmp.hexdigest()
    return tmp

def chiffrement_fichier(fichier_entree,fichier_sortie,mdp,IV):

    with open (fichier_entree,'rb') as f:
        donnees=f.read()

    cipher=AES.new(mdp.digest(),AES.MODE_CBC,IV)
    donnees_paddees= Padding.pad(donnees,16)
    donnees_chiffrees= cipher.encrypt(donnees_paddees)

    with open(fichier_sortie,'wb') as ef:
        ef.write(donnees_chiffrees)

def dechiffrement_fichier(fichier_entree,fichier_sortie,mdp,IV):

    with open (fichier_entree,'rb') as ef:
        donnees_chiffrees=ef.read()

    cipher=AES.new(mdp.digest(),AES.MODE_CBC,IV)
    donnees_paddees= cipher.decrypt(donnees_chiffrees)
    donnees= Padding.unpad(donnees_paddees,16)

    with open(fichier_sortie,'wb') as f:
        f.write(donnees)

if __name__ == '__main__':
    
    # identification
    # le master password du compte est le premier mdp du fichier passwords.txt (hash256("azerty"))
    identification = False
    masterPassword = ""
    print("Identification")
    print("Master password: ")
    masterPassword = hashlib.sha256()
    masterPassword.update(input().encode('UTF-8'))
    masterPassword=chiffrement_mp(masterPassword)
    mp = readMasterPassword()
    if(masterPassword.hexdigest() == mp):
        print("masterPassword correct")
        identification = True
    else:
        print("masterPassword incorrect")

    if identification:
        fenetre_action()


        if action == Actions.SYNC_THIS:
            sync_this_device(method)
        elif action == Actions.SYNC_OTHER:
            sync_other_device(method)
        elif action == Actions.NONE:
            print("You have to chose an action to do")