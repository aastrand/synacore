class Room:
    counter = 0

    def __init__(self, label, east=None, west=None, north=None, south=None):
        self.label = label
        self.east = east
        self.west = west
        self.north = north
        self.south = south
        self.id = Room.counter
        Room.counter += 1

    def __repr__(self):
        return f"Room({self.label}), id={self.id}"

    def __hash__(self):
        return hash(self.id)

    def __eq__(self, other):
        return self.id == other.id

    def __str__(self):
        return f"Room({self.label}), id={self.id}"


# row 1
orb = Room(22)
minus1 = Room("-")
nine = Room(9)
mul1 = Room("*")

# row 2
plus = Room("+")
four1 = Room(4)
minus2 = Room("-")
eighteen = Room(18)

# row 3
four2 = Room(4)
mul2 = Room("*")
eleven = Room(11)
mul3 = Room("*")

# roow 4
mul4 = Room("*")
eight = Room(8)
minus3 = Room("-")

vault = Room(1, west=minus3, south=mul3)

minus3.east = vault
minus3.west = eight
minus3.south = eleven

eight.east = minus3
eight.west = mul4
eight.south = mul2

mul4.east = eight
mul4.souith = four2

four2.east = mul2
four2.north = mul4
four2.south = plus

mul2.west = four2
mul2.north = eight
mul2.east = eleven
mul2.south = four1

eleven.west = mul3
eleven.north = minus3
eleven.south = minus2
eleven.east = mul2

mul3.north = vault
mul3.west = eleven
mul3.south = eighteen

eighteen.north = mul3
eighteen.south = mul1
eighteen.east = minus2

minus2.west = four1
minus2.north = eleven
minus2.south = nine
minus2.west = eighteen

four1.west = plus
four1.north = mul2
four1.south = minus1
four1.east = minus2

plus.east = four1
plus.north = four2
plus.south = orb

orb.north = plus
orb.east = minus1

minus1.west = orb
minus1.north = four1
minus1.east = nine

nine.west = minus1
nine.north = minus2
nine.east = mul1

mul1.west = nine
mul1.north = eighteen

q = [(orb, 22, None, [orb], 0)]
while q:
    cur, val, lastop, path, count = q.pop(0)
    if cur == vault or count == 12:
        if cur == vault and val == 30:
            for p in path:
                print(p.label, end=" ")
            print()
            break
        continue

    for neigh in (cur.east, cur.west, cur.north, cur.south):
        if neigh and neigh != orb:
            if lastop == "+":
                q.append((neigh, val + neigh.label, None, path + [neigh], count + 1))
            elif lastop == "-":
                q.append((neigh, val - neigh.label, None, path + [neigh], count + 1))
            elif lastop == "*":
                q.append((neigh, val * neigh.label, None, path + [neigh], count + 1))
            else:
                q.append((neigh, val, neigh.label, path + [neigh], count + 1))
