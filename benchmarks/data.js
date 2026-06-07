window.BENCHMARK_DATA = {
  "lastUpdate": 1780793838326,
  "repoUrl": "https://github.com/suonengine/suon",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "108e2aee63b42060b1910ece896227602270a468",
          "message": "chore: initial commit",
          "timestamp": "2026-06-06T17:28:54-03:00",
          "tree_id": "cfbf22c1d5a59cb90d80b233dd659039e9774f39",
          "url": "https://github.com/suonengine/suon/commit/108e2aee63b42060b1910ece896227602270a468"
        },
        "date": 1780777889576,
        "tool": "cargo",
        "benches": [
          {
            "name": "xtea/expand_key",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 4962,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5041,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5101175,
            "range": "± 11170",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5211836,
            "range": "± 15056",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1_mebibyte",
            "value": 10298308,
            "range": "± 11357",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "ec83a9598f15ae0fed8e5af45ac8865f5f35b570",
          "message": "chore: initial commit",
          "timestamp": "2026-06-06T17:49:14-03:00",
          "tree_id": "ebca10380962b5556eedfa9fe0450f6e57bd0bca",
          "url": "https://github.com/suonengine/suon/commit/ec83a9598f15ae0fed8e5af45ac8865f5f35b570"
        },
        "date": 1780779188246,
        "tool": "cargo",
        "benches": [
          {
            "name": "xtea/expand_key",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 113,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6397,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 6498,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6567327,
            "range": "± 16311",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6711453,
            "range": "± 13693",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1_mebibyte",
            "value": 13273106,
            "range": "± 43900",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "83cbdba297b263738bf8fb00da1b8e6614c101cc",
          "message": "feat(suon_xtea): make expand const fn, rewrite benchmarks with hot+cold cache, throughput, improved naming",
          "timestamp": "2026-06-06T19:48:52-03:00",
          "tree_id": "f3797c6eff75979e0661d14beefdc9d7fb088f4d",
          "url": "https://github.com/suonengine/suon/commit/83cbdba297b263738bf8fb00da1b8e6614c101cc"
        },
        "date": 1780786424103,
        "tool": "cargo",
        "benches": [
          {
            "name": "xtea/expand_key",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/24_bytes",
            "value": 214,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 372,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 735,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1430,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2843,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5853,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5913178,
            "range": "± 129789",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/24_bytes",
            "value": 205,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 366,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 733,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1452,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2885,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5942,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5923875,
            "range": "± 396067",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 371,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1441,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5846,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 406,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1602,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6575,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11772,
            "range": "± 183",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "dfeff4f1badae48d45b0ec7542323e42974604a8",
          "message": "feat(suon_rsa): pure-rust raw RSA decryption",
          "timestamp": "2026-06-06T21:10:02-03:00",
          "tree_id": "ba6549bfe337dde1a5b5585b2742bf00503c75a3",
          "url": "https://github.com/suonengine/suon/commit/dfeff4f1badae48d45b0ec7542323e42974604a8"
        },
        "date": 1780791312553,
        "tool": "cargo",
        "benches": [
          {
            "name": "rsa/load_pem",
            "value": 4866,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 106785,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 2337501,
            "range": "± 9701",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 2451584,
            "range": "± 10413",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/expand_key",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 113,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/24_bytes",
            "value": 202,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 425,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 825,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1678,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 3220,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6398,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6550519,
            "range": "± 32447",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/24_bytes",
            "value": 191,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 408,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 822,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1639,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 3267,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 6501,
            "range": "± 148",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6709474,
            "range": "± 14831",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 429,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1673,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 6403,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 410,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1698,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6511,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 12911,
            "range": "± 49",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "e6e16ce38bb0feb75baaaa3d9adcab6955596e40",
          "message": "feat(suon_rsa): pure-rust raw RSA decryption",
          "timestamp": "2026-06-06T21:15:22-03:00",
          "tree_id": "f915d7bd9edc27d1d33d6505f63dcfc52326e916",
          "url": "https://github.com/suonengine/suon/commit/e6e16ce38bb0feb75baaaa3d9adcab6955596e40"
        },
        "date": 1780791693905,
        "tool": "cargo",
        "benches": [
          {
            "name": "rsa/load_pem",
            "value": 5011,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 101199,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 2269619,
            "range": "± 34356",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 2385129,
            "range": "± 38770",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/expand_key",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/24_bytes",
            "value": 209,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 370,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 730,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1426,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2840,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5848,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5922819,
            "range": "± 109241",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/24_bytes",
            "value": 216,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 369,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 742,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1498,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2887,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5958,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5934472,
            "range": "± 19002",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 372,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1432,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5857,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 367,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1454,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 5962,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11796,
            "range": "± 65",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "committer": {
            "email": "ramonbernardo.contato@gmail.com",
            "name": "ramon-bernardo",
            "username": "ramon-bernardo"
          },
          "distinct": true,
          "id": "acfb13cae77c5d9ab1f175c94b8f4842ae0b79a6",
          "message": "refactor(suon_rsa): remove pem/pkcs1 deps, manual PEM/DER parsing, CRT decrypt, tests",
          "timestamp": "2026-06-06T21:52:11-03:00",
          "tree_id": "1dd8b287fd4efc9cfcdb68b47347b3faf9e03429",
          "url": "https://github.com/suonengine/suon/commit/acfb13cae77c5d9ab1f175c94b8f4842ae0b79a6"
        },
        "date": 1780793838109,
        "tool": "cargo",
        "benches": [
          {
            "name": "rsa/load_pem",
            "value": 2451,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 105952,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 694057,
            "range": "± 8086",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 800037,
            "range": "± 8945",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/expand_key",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/8_bytes",
            "value": 113,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/24_bytes",
            "value": 202,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 425,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 825,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1669,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 3219,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6398,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6587281,
            "range": "± 41137",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/8_bytes",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/24_bytes",
            "value": 191,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 408,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 822,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1640,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 3268,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 6499,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6747440,
            "range": "± 260245",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 429,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1674,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 6403,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 410,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1699,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6508,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 12901,
            "range": "± 38",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}