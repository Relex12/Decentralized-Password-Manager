import argparse
import asyncio
from typing import Any
import aioice
from aioice.candidate import Candidate
import time
from aioice.ice import CandidatePair

SERVEUR_STUN = ("stun1.l.google.com", 19302)
fichier_mdp_chiffres='passwords_chiffres.txt'

def sync_this_device_Ice():
    async def main():
        conn_a = aioice.Connection(ice_controlling=True, stun_server=SERVEUR_STUN)
        conn_b = aioice.Connection(ice_controlling=False, stun_server=SERVEUR_STUN)

        # les candidats distants de b sont les locaux de a
        await conn_a.gather_candidates()

        for i in range(len(conn_a.local_candidates)):
            await conn_b.add_remote_candidate(Candidate.from_sdp(conn_a.local_candidates[i].to_sdp()))

        print (conn_b.remote_candidates)

        conn_b.remote_username = conn_a.local_username
        conn_b.remote_password = conn_a.local_password

        # respectivement
        await conn_b.gather_candidates()
        for j in range(len(conn_b.local_candidates)):
            await conn_a.add_remote_candidate(Candidate.from_sdp(conn_b.local_candidates[j].to_sdp()))

        print (conn_a.remote_candidates)
        conn_a.remote_username = conn_b.local_username
        conn_a.remote_password = conn_b.local_password

        # connection
        await asyncio.gather(conn_a.connect(), conn_b.connect())


        # envoie de données de b vers a
        with open(fichier_mdp_chiffres,'rb') as ef:
            donnees=ef.read()
        await conn_b.send(donnees)

        data = await conn_a.recv()
        with open (fichier_mdp_chiffres,'wb') as ef:
            ef.write(data)
        print('Nous avons reçu le fichier')

        # fermeture de la connexion
        await asyncio.gather(conn_a.close(), conn_b.close())
    
    asyncio.get_event_loop().run_until_complete(main())



def sync_other_device_Ice():
    async def main():
        conn_a = aioice.Connection(ice_controlling=True, stun_server=SERVEUR_STUN)
        conn_b = aioice.Connection(ice_controlling=False, stun_server=SERVEUR_STUN)

        # les candidats distants de b sont les locaux de a
        await conn_a.gather_candidates()

        for i in range(len(conn_a.local_candidates)):
            await conn_b.add_remote_candidate(Candidate.from_sdp(conn_a.local_candidates[i].to_sdp()))

        print (conn_b.remote_candidates)

        conn_b.remote_username = conn_a.local_username
        conn_b.remote_password = conn_a.local_password

        # respectivement
        await conn_b.gather_candidates()
        for j in range(len(conn_b.local_candidates)):
            await conn_a.add_remote_candidate(Candidate.from_sdp(conn_b.local_candidates[j].to_sdp()))

        print (conn_a.remote_candidates)
        conn_a.remote_username = conn_b.local_username
        conn_a.remote_password = conn_b.local_password

        # connection
        await asyncio.gather(conn_a.connect(), conn_b.connect())


        # envoie de données de a vers b
        with open(fichier_mdp_chiffres,'rb') as ef:
            donnees=ef.read()
        await conn_a.send(donnees)

        data = await conn_b.recv()
        with open (fichier_mdp_chiffres,'wb') as ef:
            ef.write(data)
        print("L'appareil distant a reçu le fichier")

        # fermeture de la connexion
        await asyncio.gather(conn_a.close(), conn_b.close())

    asyncio.get_event_loop().run_until_complete(main())