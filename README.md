# hCaptcha WASM deobfuscator & fetcher

## Important
- hCaptcha appears to randomly choose between XOR and ChaCha20 for memory encryption. Only XOR is currently supported—ChaCha20 is not yet implemented.
- The output WASM is fully runnable, but you must comment out the memory block initialization call in the JS. 

## Required JS modification
```js
        Af.then(function(OE) {
            return function(OE, hW) {
                return new Promise(function(vk, hn) {
                    WebAssembly.instantiate(OE, hW).then(function(hW) {
                        if (!hW || !hW.exports)
                            throw new Error("Failed to instantiate");
                        hW instanceof WebAssembly.Instance ? vk({
                            instance: hW,
                            module: OE
                        }) : vk(hW)
                    }).catch(function(OE) {
                        return hn(OE)
                    })
                }
                )
            }(OE, {
                a: sf
            })
        }).then(function(hW) {
            !function(OE) {
                qy = OE;
                for (hW = Math[N_(332)]((qy.vb[N_(333)][N_(334)] - qi) / rR),
                vk = 0,
                void 0; vk < hW; vk++) {
                    var hW;
                    var vk;
                    // qy.Ab(vk) <-- COMMENT THIS
                }
            }(hW.instance.exports),
            OE()
        }).catch(function(OE) {
            return hW(OE)
        })
```

## Features
- Revert memory encryption (xor)
- Fetch events string

## Dependencies
- [Walrus](https://github.com/rustwasm/walrus) - WASM transformations
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling