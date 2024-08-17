from bloom import BloomFilter

bf = BloomFilter(100_000_000, 0.001, "example.db")

bf.add("alice")
bf.add("bob")

print(bf.contains("alice"))  # True
print(bf.contains("bob"))  # True
print(bf.contains("charlie"))  # False
