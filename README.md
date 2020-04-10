# slatepack
This is a test repo to experiment with a standard for generating armored slates for Grin transactions.

It should be able to:

- [x]  Accept json slate strings prepared in the same format as in `grin-wallet` and return an armored slate string

- [ ]  Accept an armored slate string prepared with `slatepack` and return a json slate string compatible with `grin-wallet`

### How slatepack works when serializing:
1. Converts slate string to binary
2. Generates an error checking code from the slate binary
3. Concatenates error checking code bytes and slate bytes
4. Base58 encodes the output from step 3
5. Formats the output from step 4 into 15 character words for readability
6. Frames the encoded payload with human and machine readable characters

#### V4 initial slate json string:
```
{\n  \"ver\": \"4.3\",\n  \"sta\": \"S1\",\n  \"id\": \"mavTAjNm4NVztDwh4gdSrQ\",\n  \"amt\": \"1000000000\",\n  \"fee\": \"8000000\",\n  \"sigs\": [\n    {\n      \"excess\": \"02ef37e5552a112f829e292bb031b484aaba53836cd6415aacbdb8d3d2b0c4bb9a\",\n      \"nonce\": \"03c78da42179b80bd123f5a39f5f04e2a918679da09cad5558ebbe20953a82883e\"\n    }\n  ]\n}\n
```

#### V4 initial slate armored with slatepack:
```
BEGIN SLATEPACK. 2xfX9bS82gxJ6jN D6X4X843wrT84DT FkstYawTtacDqeU HybZLwcF26YXCix bmpTcw3hii6BF4x axfussSBrZq7xMQ P1rbw3GpebXkMeY i7aSjRZgxqDJwzt MyGqBauGHxEFZNg FeEbVFsqXaKkKwK PQdxrpKutVmJV67 pbY4nbeZgPtRaZj QZL61Wj7iGqKBuu tDvEwUBsuhb9GRf 1MK3jegnbKG5JJr QVrYignWoZrpXUx PiDobVMLh7RTRrz T6GNKJftiwJ5gup f7T69mFG9H8JqCG A4i5ogfcHhfgg5b 2AzBJA49nh39Pyh zotpGBj7a7RK4Kr bWqksP7iTxvfdUB zVwinrRjLeryvF7 uroTKm514ZDDrKf ZbyncaZXcFGYHWM tWp5ccsjjtM1JqB adragavjHQyjqkU 2JH9YnoRkx2AyuU qvn7nnb4fMTAVSw sbAPwBTua7njNht nhzRqtdHTr9KM9q eXHDN9iascu. END SLATEPACK.
```

### Example use:
- Copy slate json into file named `v4_test.slate`

```
{
  "ver": "4.3",
  "sta": "S1",
  "id": "mavTAjNm4NVztDwh4gdSrQ",
  "amt": "1000000000",
  "fee": "8000000",
  "sigs": [
    {
      "excess": "02ef37e5552a112f829e292bb031b484aaba53836cd6415aacbdb8d3d2b0c4bb9a",
      "nonce": "03c78da42179b80bd123f5a39f5f04e2a918679da09cad5558ebbe20953a82883e"
    }
  ]
}

```

- Add slatepack dependency to `Cargo.toml`

```
[dependencies]
slatepack = { git = "https://github.com/j01tz/slatepack" }
```

```
use slatepack;

let slate = include_str!("v4_test.slate");
let armored_slate = slatepack::armor(&slate).unwrap();
println!("{:?}", armored_slate);
```

This should not be used for any real transactions. It is for educational purposes to visualize and experiment with the armored slate string format.
