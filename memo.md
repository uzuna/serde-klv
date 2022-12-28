depth:0

c[depth].push(v)

L = len(c[depth])
V = c[depth]

depth
- UniversalKey
- LVSeq
- Value

struct -> 2depth
- KLV
    - V: [KLV]
        - V
tuple -> 1depth
- KLV
    - [V]
seq -> 1depth
- KLV
    - [V]

現在
V.serialize => V in cache
StructSerializer -> KLV in output
これを V in cacheに書き換える
to_bytes TopLevelが

structの期待値
- key
- [key, value]

愚直に作るならKV

```rust

KV {
    key: &[u8],
    value: Vec<KV>
}

impl KV {
    fn to_byte() -> Vec<u8>{

    }
}

```

serialize_struct = KLVを期待している
- structの名前をとるのに必須
ただし二階層目からはVになってほしい

Vスタイルを先に書いてdepth0の時だけ特別な動きをする実装を考える

