import secrets
from math import gcd
import hmac, hashlib

# Paramètres de démonstration (en pratique choisir un p grand et sûr)
p = 2089      # p premier -> (Z/pZ)* est cyclique d'ordre q = p-1
g = 2         # générateur (démonstration)
q = p - 1     # ordre du groupe multiplicatif - l'arithmétique des exposants = mod q

def rand_nonzero(modulus: int) -> int:
    """Tire uniformément un entier dans {1,2,...,modulus-1} (non nul)"""
    while True:
        x = secrets.randbelow(modulus)
        if x != 0:
            return x

def rand_coprime(modulus: int) -> int:
    """Tire un entier 1..modulus-1 premier avec 'modulus' (pour garantir l'inverse modulaire)"""
    while True:
        x = rand_nonzero(modulus)
        if gcd(x, modulus) == 1:
            return x

def modinv(a: int, m: int) -> int:
    """Inverse modulaire de a (mod m). Lève ValueError s'il n'existe pas"""
    return pow(a, -1, m)

def int_to_bytes(x: int) -> bytes:
    """Conversion d'un entier non négatif vers une chaîne d'octets big-endian minimale"""
    if x == 0:
        return b"\x00"
    return x.to_bytes((x.bit_length() + 7)//8, "big")

class User:
    def __init__(self, name: str, x: int, y: int):
        self.name = name   # identifiant textuel (utilisé dans la balise HMAC)
        self.x = x         # clé longue durée (privée)
        self.y = y         # clé publique y = g^x mod p

def setup_user(name: str) -> User:
    """
    Génère une paire de clés longue durée
    Exige x copremier avec q pour assurer l'existence de x^{-1} (mod q)
    """
    x = rand_coprime(q)
    y = pow(g, x % q, p)
    return User(name, x, y)

# HMAC-SHA256 - confirmation 
def compute_tag(name: str, K: int, transcript: bytes) -> str:
    """
    Calcule une balise d'authentification: tag = HMAC-SHA256( key = K (en octets), msg = name | transcript )
    Le 'transcript' regroupe les éléments publics échangés pendant l'étape 1
    """
    key = int_to_bytes(K % p)
    msg = name.encode("utf-8") + b"|" + transcript
    return hmac.new(key, msg=msg, digestmod=hashlib.sha256).hexdigest()

def verify_tag(name: str, K: int, transcript: bytes, tag_hex: str) -> bool:
    """Vérifie une balise HMAC attendue pour (name, K, transcript)."""
    exp = compute_tag(name, K, transcript)
    return hmac.compare_digest(exp, tag_hex)

def three_party_dh_with_confirmation(A: User, B: User, C: User):
    """
    Étape 1 : chaque partie choisit un exposant éphémère (a,b,c) et envoie ses T_..
    Étape 2 : chacun calcule localement la clé K 
    Confirmation : HMAC-SHA256 avec K sur le transcript public
    Retourne : (K_A, K_B, K_C, tags, all_ok)
    """
    # Étape 1 
    a = rand_nonzero(q)  # exposants éphémères (secrets temporaires)
    b = rand_nonzero(q)
    c = rand_nonzero(q)

    # Messages 
    # A -> B, A -> C
    T_AB = pow(B.y, (a * A.x) % q, p)
    T_AC = pow(C.y, (a * A.x) % q, p)
    # B -> A, B -> C
    T_BA = pow(A.y, (b * B.x) % q, p)
    T_BC = pow(C.y, (b * B.x) % q, p)
    # C -> A, C -> B
    T_CA = pow(A.y, (c * C.x) % q, p)
    T_CB = pow(B.y, (c * C.x) % q, p)

    # Étape 2
    inv_xA = modinv(A.x % q, q)
    inv_xB = modinv(B.x % q, q)
    inv_xC = modinv(C.x % q, q)

    # Alice :
    g_axA_A = pow(A.y, a % q, p)        # g^{a x_A}
    g_bxB_A = pow(T_BA, inv_xA, p)      # (g^{b x_B x_A})^{x_A^{-1}} = g^{b x_B}
    g_cxC_A = pow(T_CA, inv_xA, p)      # g^{c x_C}
    K_A = (g_axA_A * g_bxB_A * g_cxC_A) % p

    # Bob :
    g_axA_B = pow(T_AB, inv_xB, p)
    g_bxB_B = pow(B.y, b % q, p)
    g_cxC_B = pow(T_CB, inv_xB, p)
    K_B = (g_axA_B * g_bxB_B * g_cxC_B) % p

    # Charlie :
    g_axA_C = pow(T_AC, inv_xC, p)
    g_bxB_C = pow(T_BC, inv_xC, p)
    g_cxC_C = pow(C.y, c % q, p)
    K_C = (g_axA_C * g_bxB_C * g_cxC_C) % p

    # Confirmation de clé  
    transcript = (
        f"p={p},g={g},q={q},yA={A.y},yB={B.y},yC={C.y},"
        f"T_AB={T_AB},T_AC={T_AC},T_BA={T_BA},T_BC={T_BC},T_CA={T_CA},T_CB={T_CB}"
    ).encode("utf-8")

    tag_A = compute_tag(A.name, K_A, transcript)
    tag_B = compute_tag(B.name, K_B, transcript)
    tag_C = compute_tag(C.name, K_C, transcript)

    # Vérifications croisées 
    A_ok = verify_tag(B.name, K_A, transcript, tag_B) and verify_tag(C.name, K_A, transcript, tag_C)
    B_ok = verify_tag(A.name, K_B, transcript, tag_A) and verify_tag(C.name, K_B, transcript, tag_C)
    C_ok = verify_tag(A.name, K_C, transcript, tag_A) and verify_tag(B.name, K_C, transcript, tag_B)
    all_ok = A_ok and B_ok and C_ok

    tags = {"A": tag_A, "B": tag_B, "C": tag_C}
    return K_A, K_B, K_C, tags, all_ok

if __name__ == "__main__":
    # Clés longue durée
    A = setup_user("Alice")
    B = setup_user("Bob")
    C = setup_user("Charlie")

    K_A, K_B, K_C, tags, ok = three_party_dh_with_confirmation(A, B, C)

    print("A public y_A =", A.y)
    print("B public y_B =", B.y)
    print("A public y_C =", C.y)
    print()
    print("Clés de session :")
    print("\tK_A = ", K_A)
    print("\tK_B = ", K_B)
    print("\tK_C = ", K_C)
    print()
    print("Tags (hex): ", tags)
    print()
