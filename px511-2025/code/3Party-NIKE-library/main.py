from bplib import bp
from petlib.bn import Bn

def random_scalar(order: Bn) -> Bn:
    """Return a random scalar in [1, order-1]."""
    rnd = order.random()
    if rnd == 0:
        return random_scalar(order)
    return rnd

def main():
    # --- Setup bilinear pairing group ---
    G = bp.BpGroup()
    g1, g2 = G.gen1(), G.gen2()
    order = G.order()

    print("[*] Group order (bits):", order.num_bits())

    # Secrets for Alice, Bob, Charlie (Bn)
    a = random_scalar(order)
    b = random_scalar(order)
    c = random_scalar(order)

    print(f"a = {int(a)}")
    print(f"b = {int(b)}")
    print(f"c = {int(c)}")

    # Public points
    A1 = g1 * a
    B1 = g1 * b
    C1 = g1 * c

    A2 = g2 * a
    B2 = g2 * b
    C2 = g2 * c

    # Alice: K_A = e(B1, C2)^a
    e_BC = G.pair(B1, C2)
    K_A = e_BC ** a

    # Bob: K_B = e(C1, A2)^b
    e_CA = G.pair(C1, A2)
    K_B = e_CA ** b

    # Charlie: K_C = e(A1, B2)^c
    e_AB = G.pair(A1, B2)
    K_C = e_AB ** c

    print("\n[*] Checking equality in GT...")
    print("K_A == K_B ?", K_A == K_B)
    print("K_B == K_C ?", K_B == K_C)
    print("K_A == K_C ?", K_A == K_C)

    if K_A == K_B == K_C:
        print("\n[+] Success: all three parties share the same pairing-based key.")
    else:
        print("\n[-] Keys do not match. Something went wrong.")

    key_bytes = K_A.export()
    print("\nSession key (GT element, hex):")
    print(key_bytes.hex())

if __name__ == "__main__":
    main()