window.BENCHMARK_DATA = {
  "lastUpdate": 1781307468170,
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
          "id": "f8283e88b3058f6766bdb75876fc6cd5ba1c7c5f",
          "message": "feat(suon_resource): init crate with Resource trait, Resources container, benches",
          "timestamp": "2026-06-07T10:04:01-03:00",
          "tree_id": "bbf2a9a9e9a151a20de9cc3c182e1935770fe7c2",
          "url": "https://github.com/suonengine/suon/commit/f8283e88b3058f6766bdb75876fc6cd5ba1c7c5f"
        },
        "date": 1780839400000,
        "tool": "cargo",
        "benches": [
          {
            "name": "resource/init",
            "value": 63,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 11,
            "range": "± 0",
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
          "id": "b5074bbd53b433f5a857f02b1360456d497aeada",
          "message": "feat(suon_resource): init crate with Resource trait, Resources container, benches",
          "timestamp": "2026-06-07T10:04:01-03:00",
          "tree_id": "595a523a28f082cefe862852ab4b64509bb92b45",
          "url": "https://github.com/suonengine/suon/commit/b5074bbd53b433f5a857f02b1360456d497aeada"
        },
        "date": 1780837816282,
        "tool": "cargo",
        "benches": [
          {
            "name": "resource_init",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource_insert",
            "value": 134,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource_get",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource_get_mut",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2464,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 105891,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 697091,
            "range": "± 2199",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 803380,
            "range": "± 12711",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 425,
            "range": "± 20",
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
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 3218,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6400,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6554026,
            "range": "± 170420",
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
            "range": "± 6",
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
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1640,
            "range": "± 2",
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
            "value": 6500,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6706492,
            "range": "± 199346",
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
            "value": 1672,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 6404,
            "range": "± 54",
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
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6507,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 12904,
            "range": "± 601",
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
          "id": "f8283e88b3058f6766bdb75876fc6cd5ba1c7c5f",
          "message": "feat(suon_resource): init crate with Resource trait, Resources container, benches",
          "timestamp": "2026-06-07T10:39:47-03:00",
          "tree_id": "14717204c395ba9175b2ed83771cca716c6f379c",
          "url": "https://github.com/suonengine/suon/commit/f8283e88b3058f6766bdb75876fc6cd5ba1c7c5f"
        },
        "date": 1780840131889,
        "tool": "cargo",
        "benches": [
          {
            "name": "resource/init",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2666,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 101175,
            "range": "± 3506",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 680582,
            "range": "± 6987",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 783016,
            "range": "± 8155",
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
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 370,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 729,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1426,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2840,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5847,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5919289,
            "range": "± 18017",
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
            "value": 215,
            "range": "± 0",
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
            "value": 741,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1564,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2887,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5958,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5918450,
            "range": "± 11257",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 373,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1466,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5864,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 367,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1453,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 5950,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11789,
            "range": "± 559",
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
          "id": "c2b5cf1e24e54eaf7e40329118988cd7fb125bd3",
          "message": "feat(suon_channel): add channel crate with IntoTask and derive",
          "timestamp": "2026-06-07T12:18:24-03:00",
          "tree_id": "86e81b672509c1249cb7b55b7fbbe29e9b1917f3",
          "url": "https://github.com/suonengine/suon/commit/c2b5cf1e24e54eaf7e40329118988cd7fb125bd3"
        },
        "date": 1780845909037,
        "tool": "cargo",
        "benches": [
          {
            "name": "channel/send",
            "value": 104,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 125,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 197,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 655,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4284,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 32840,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 141,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2686,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 101068,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 681296,
            "range": "± 1321",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 798011,
            "range": "± 7846",
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
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1427,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2841,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5850,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5889337,
            "range": "± 133818",
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
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 369,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 741,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1573,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2887,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5958,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5923514,
            "range": "± 10850",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 373,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1433,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5855,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 367,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1454,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 5948,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11796,
            "range": "± 124",
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
          "id": "08257b3a26c213a1b7d44ea8c4d6b2c24d1aaa56",
          "message": "feat(suon_channel): add channel crate with IntoTask and derive",
          "timestamp": "2026-06-07T12:25:38-03:00",
          "tree_id": "9c1047125fb2a4c76da4b86d04000c5154ae8ffd",
          "url": "https://github.com/suonengine/suon/commit/08257b3a26c213a1b7d44ea8c4d6b2c24d1aaa56"
        },
        "date": 1780846358866,
        "tool": "cargo",
        "benches": [
          {
            "name": "channel/send",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 125,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 186,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 679,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4505,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 34507,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 133,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 83,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2465,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 106091,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 693074,
            "range": "± 11807",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 801896,
            "range": "± 2355",
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
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 425,
            "range": "± 2",
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
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 3221,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6396,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6544729,
            "range": "± 24220",
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
            "value": 195,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 408,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 822,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1640,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 3265,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 6496,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6706920,
            "range": "± 43097",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 429,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1669,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 6404,
            "range": "± 25",
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
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6507,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 12896,
            "range": "± 237",
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
          "id": "c6de025bbed48f6f93dc03d74cdf72c7bc9030ea",
          "message": "feat(suon_app): create app crate with lifecycle and plugin system",
          "timestamp": "2026-06-07T16:43:07-03:00",
          "tree_id": "184f1a29fbdc62e77351b38e8ee0300600cc3df4",
          "url": "https://github.com/suonengine/suon/commit/c6de025bbed48f6f93dc03d74cdf72c7bc9030ea"
        },
        "date": 1780861822180,
        "tool": "cargo",
        "benches": [
          {
            "name": "app/empty_shutdown",
            "value": 236,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "app/startup_system",
            "value": 298,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "app/task_dispatch_100",
            "value": 2078,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send",
            "value": 113,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 102,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 195,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 652,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4236,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 32813,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 140,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2649,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 101292,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 680686,
            "range": "± 10105",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 782744,
            "range": "± 6805",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 370,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 728,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1427,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2840,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5853,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5918864,
            "range": "± 49585",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 369,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 742,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1580,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2886,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5961,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5920082,
            "range": "± 10318",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 372,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1433,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5856,
            "range": "± 38",
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
            "value": 1453,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 5948,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11794,
            "range": "± 43",
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
          "id": "75bf0da91e7ea5aae63fe6a09373051b4d06678b",
          "message": "feat(suon_app): create app crate with lifecycle and plugin system",
          "timestamp": "2026-06-07T16:51:17-03:00",
          "tree_id": "845ae69d999e7652437ebcf4a356892468e7a943",
          "url": "https://github.com/suonengine/suon/commit/75bf0da91e7ea5aae63fe6a09373051b4d06678b"
        },
        "date": 1780862335187,
        "tool": "cargo",
        "benches": [
          {
            "name": "app/empty_shutdown",
            "value": 238,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "app/startup_system",
            "value": 274,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "app/task_dispatch_100",
            "value": 2110,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send",
            "value": 105,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 100,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 191,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 653,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4234,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 32681,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 135,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2656,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 101626,
            "range": "± 937",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 681712,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 784992,
            "range": "± 8708",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/128_bytes",
            "value": 730,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256_bytes",
            "value": 1427,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 2839,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 5853,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 5913785,
            "range": "± 22480",
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
            "value": 215,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64_bytes",
            "value": 369,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 741,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1573,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 2887,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 5961,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 5921842,
            "range": "± 14593",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 372,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1436,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 5860,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 367,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1454,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 5948,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 11791,
            "range": "± 84",
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
          "id": "72477bbb6d7df55b8fd8d4f07422e013ec515ed4",
          "message": "feat(suon_lua): add Lua scripting crate with plugin architecture",
          "timestamp": "2026-06-07T20:23:49-03:00",
          "tree_id": "00fb43a2416ac3d7d12ea463b9c84e276c8c0746",
          "url": "https://github.com/suonengine/suon/commit/72477bbb6d7df55b8fd8d4f07422e013ec515ed4"
        },
        "date": 1780875152796,
        "tool": "cargo",
        "benches": [
          {
            "name": "app/empty_shutdown",
            "value": 235,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "app/startup_system",
            "value": 289,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "app/task_dispatch_100",
            "value": 1971,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send",
            "value": 102,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 108,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 186,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 674,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4485,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 34421,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 133,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "lua/vm_creation",
            "value": 42867,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "lua/expression_eval",
            "value": 2550,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "lua/store_restore_call",
            "value": 856,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_no_events",
            "value": 4786,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_with_events",
            "value": 345,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 73,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 84,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2540,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 106136,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 695611,
            "range": "± 1998",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 802416,
            "range": "± 1676",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64_bytes",
            "value": 425,
            "range": "± 25",
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
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/512_bytes",
            "value": 3220,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1024_bytes",
            "value": 6398,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/1_mebibyte",
            "value": 6549089,
            "range": "± 18124",
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
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/128_bytes",
            "value": 822,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256_bytes",
            "value": 1639,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/512_bytes",
            "value": 3266,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1024_bytes",
            "value": 6501,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/1_mebibyte",
            "value": 6712280,
            "range": "± 39211",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/64_bytes",
            "value": 428,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/256_bytes",
            "value": 1673,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/cold_cache/1024_bytes",
            "value": 6405,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/64_bytes",
            "value": 410,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/256_bytes",
            "value": 1698,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/cold_cache/1024_bytes",
            "value": 6506,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/roundtrip/1024_bytes",
            "value": 12903,
            "range": "± 407",
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
          "id": "4540e3d8c24522a62237cb5e47a9ee5b0db0bdd4",
          "message": "fix(suon_rsa): enable u64_digit feature for num-bigint-dig to fix CI build on nightly",
          "timestamp": "2026-06-11T19:47:32-03:00",
          "tree_id": "4aec0b3f3cbc2cee6422a6a7f64b10c9dcc46100",
          "url": "https://github.com/suonengine/suon/commit/4540e3d8c24522a62237cb5e47a9ee5b0db0bdd4"
        },
        "date": 1781218679072,
        "tool": "cargo",
        "benches": [
          {
            "name": "adler32/0_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 213,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 438,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 888,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1788,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 3590,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 7194,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 14400,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 28820,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 57711,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 115807,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 231861,
            "range": "± 1026",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 466639,
            "range": "± 3137",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 936946,
            "range": "± 8486",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1888742,
            "range": "± 20316",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 3904206,
            "range": "± 45959",
            "unit": "ns/iter"
          },
          {
            "name": "app/empty_shutdown",
            "value": 246,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "app/startup_system",
            "value": 432,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "app/task_dispatch_100",
            "value": 2019,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send",
            "value": 106,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 108,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_2",
            "value": 119,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4",
            "value": 140,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 188,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_16",
            "value": 264,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_32",
            "value": 417,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 688,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_128",
            "value": 1221,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_256",
            "value": 2339,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4550,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1024",
            "value": 8890,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_2048",
            "value": 17607,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 35102,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8192",
            "value": 70072,
            "range": "± 1276",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_16384",
            "value": 140289,
            "range": "± 2190",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_32768",
            "value": 280895,
            "range": "± 4596",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_65536",
            "value": 558637,
            "range": "± 3296",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 134,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "lua/vm_creation",
            "value": 43755,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "lua/expression_eval",
            "value": 2616,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "lua/store_restore_call",
            "value": 910,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_no_events",
            "value": 4808,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_with_events",
            "value": 332,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "resource/init",
            "value": 72,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "resource/insert",
            "value": 93,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "resource/get_mut",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/load_pem",
            "value": 2490,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/encrypt/1024_bit",
            "value": 67150,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/decrypt/1024_bit",
            "value": 301344,
            "range": "± 7329",
            "unit": "ns/iter"
          },
          {
            "name": "rsa/roundtrip/1024_bit",
            "value": 368784,
            "range": "± 2205",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/expand_key",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/0_bytes",
            "value": 0,
            "range": "± 0",
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
          "id": "cde61bf3c7e1d89a383040058a661060fdbaa5e0",
          "message": "feat(suon_network): add complete networking crate with TCP/HTTP protocol, connection manager, and REST API",
          "timestamp": "2026-06-11T19:59:32-03:00",
          "tree_id": "4b7a4f318206bc1e6f3b6d4c71c8deb6f06619f1",
          "url": "https://github.com/suonengine/suon/commit/cde61bf3c7e1d89a383040058a661060fdbaa5e0"
        },
        "date": 1781220949124,
        "tool": "cargo",
        "benches": [
          {
            "name": "adler32/0_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 101,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 213,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 438,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 888,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1787,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 3589,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 7189,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 14394,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 28814,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 57706,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 115707,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 231354,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 466189,
            "range": "± 6893",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 950439,
            "range": "± 30228",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1999732,
            "range": "± 19319",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 3975625,
            "range": "± 121562",
            "unit": "ns/iter"
          },
          {
            "name": "app/empty_shutdown",
            "value": 232,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "app/startup_system",
            "value": 334,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "app/task_dispatch_100",
            "value": 2048,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send",
            "value": 103,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1",
            "value": 108,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_2",
            "value": 120,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4",
            "value": 142,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8",
            "value": 187,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_16",
            "value": 263,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_32",
            "value": 410,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_64",
            "value": 679,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_128",
            "value": 1217,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_256",
            "value": 2330,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_512",
            "value": 4539,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_1024",
            "value": 8870,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_2048",
            "value": 17563,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_4096",
            "value": 34976,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_8192",
            "value": 69679,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_16384",
            "value": 139369,
            "range": "± 1491",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_32768",
            "value": 278958,
            "range": "± 1989",
            "unit": "ns/iter"
          },
          {
            "name": "channel/drain_65536",
            "value": 557792,
            "range": "± 3019",
            "unit": "ns/iter"
          },
          {
            "name": "channel/send_and_drain",
            "value": 139,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "lua/vm_creation",
            "value": 43473,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "lua/expression_eval",
            "value": 2614,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "lua/store_restore_call",
            "value": 865,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_no_events",
            "value": 4823,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "lua/dispatch_with_events",
            "value": 345,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/0_bytes",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/2_bytes",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/4_bytes",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/8_bytes",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/16_bytes",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/32_bytes",
            "value": 28,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/64_bytes",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/128_bytes",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/256_bytes",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/512_bytes",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/1_kb",
            "value": 64,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/2_kb",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/4_kb",
            "value": 187,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/8_kb",
            "value": 276,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/16_kb",
            "value": 587,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/32_kb",
            "value": 838,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "connection/send/64_kb",
            "value": 1570,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "manager/register",
            "value": 363,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "manager/register_unregister",
            "value": 165,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "manager/active_connections_100",
            "value": 6108,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "manager/concurrent_register",
            "value": 241532,
            "range": "± 87519",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/0_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 214,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 438,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 888,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1788,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 3591,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 7193,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 14398,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 28824,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 57727,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 115796,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 231297,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 464561,
            "range": "± 2310",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 935118,
            "range": "± 11213",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1893747,
            "range": "± 64476",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 3862261,
            "range": "± 47207",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/0_bytes",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/2_bytes",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/4_bytes",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/8_bytes",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/16_bytes",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/32_bytes",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/64_bytes",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/128_bytes",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/256_bytes",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/512_bytes",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/1_kb",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/2_kb",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/4_kb",
            "value": 200,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/8_kb",
            "value": 305,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/16_kb",
            "value": 605,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/32_kb",
            "value": 1060,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/64_kb",
            "value": 2051,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/128_kb",
            "value": 4087,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/256_kb",
            "value": 8128,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/512_kb",
            "value": 14204,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_pad/1_mb",
            "value": 27973,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/0_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/2_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/4_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/8_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/16_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/32_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/64_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/128_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/256_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/512_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/1_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/2_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/4_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/8_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/16_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/32_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/64_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/128_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/256_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/512_kb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_unpad/1_mb",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/0_bytes",
            "value": 123,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/2_bytes",
            "value": 150,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/4_bytes",
            "value": 149,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/8_bytes",
            "value": 253,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/16_bytes",
            "value": 244,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/32_bytes",
            "value": 332,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/64_bytes",
            "value": 476,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/128_bytes",
            "value": 866,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/256_bytes",
            "value": 1633,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/512_bytes",
            "value": 3345,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/1_kb",
            "value": 6217,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/2_kb",
            "value": 12438,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/4_kb",
            "value": 25074,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/8_kb",
            "value": 49727,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/16_kb",
            "value": 99001,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/32_kb",
            "value": 197548,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/64_kb",
            "value": 396112,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/128_kb",
            "value": 791134,
            "range": "± 13998",
            "unit": "ns/iter"
          },
          {
            "name": "xtea_decrypt/256_kb",
            "value": 1577308,
            "range": "± 19893",
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
          "id": "6fa4da679f5766e6610c328200f4ccc1765da7f7",
          "message": "chore(deps): add tracing and tracing-subscriber dependencies",
          "timestamp": "2026-06-11T21:09:53-03:00",
          "tree_id": "10280429ab04ceedb4b72f7355fbce08d6216802",
          "url": "https://github.com/suonengine/suon/commit/6fa4da679f5766e6610c328200f4ccc1765da7f7"
        },
        "date": 1781223374792,
        "tool": "cargo",
        "benches": [
          {
            "name": "adler32/0_bytes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 189,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 391,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 789,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1586,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 3178,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 6363,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 12741,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 25516,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 51155,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 102776,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 207868,
            "range": "± 1974",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 422838,
            "range": "± 4978",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 900540,
            "range": "± 24878",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1800533,
            "range": "± 30329",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 3503700,
            "range": "± 41768",
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
          "id": "e73955b750faee5a80ab211676fd89b355264f32",
          "message": "refactor(channel): add pending counter, remove panic, pre-size drain buffer",
          "timestamp": "2026-06-11T21:30:59-03:00",
          "tree_id": "4c3926b2e772b53c7448644cf1b4c686e517e1b9",
          "url": "https://github.com/suonengine/suon/commit/e73955b750faee5a80ab211676fd89b355264f32"
        },
        "date": 1781224597792,
        "tool": "cargo",
        "benches": [
          {
            "name": "adler32/0_bytes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 101,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 214,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 438,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 888,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1788,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 3590,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 7191,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 14404,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 28822,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 57732,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 127866,
            "range": "± 6302",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 246023,
            "range": "± 8253",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 466977,
            "range": "± 3614",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 941628,
            "range": "± 12828",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1894887,
            "range": "± 21597",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 3866479,
            "range": "± 46287",
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
          "id": "fd68d38fa5c79129d3fc790c9d7e4c0d35fbf718",
          "message": "feat(suon_network): add ConnectionManager::stats()",
          "timestamp": "2026-06-12T20:27:17-03:00",
          "tree_id": "94f6273315386fb37e285c751b591ba897918d52",
          "url": "https://github.com/suonengine/suon/commit/fd68d38fa5c79129d3fc790c9d7e4c0d35fbf718"
        },
        "date": 1781307467567,
        "tool": "cargo",
        "benches": [
          {
            "name": "adler32/0_bytes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_bytes",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_bytes",
            "value": 4,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_bytes",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_bytes",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_bytes",
            "value": 55,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_bytes",
            "value": 123,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_bytes",
            "value": 248,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_bytes",
            "value": 521,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_bytes",
            "value": 1051,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_kb",
            "value": 2109,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/2_kb",
            "value": 4168,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/4_kb",
            "value": 8369,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/8_kb",
            "value": 16898,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/16_kb",
            "value": 33963,
            "range": "± 1297",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/32_kb",
            "value": 68307,
            "range": "± 3534",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/64_kb",
            "value": 133553,
            "range": "± 9203",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/128_kb",
            "value": 264133,
            "range": "± 14803",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/256_kb",
            "value": 525190,
            "range": "± 9562",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/512_kb",
            "value": 1065951,
            "range": "± 53387",
            "unit": "ns/iter"
          },
          {
            "name": "adler32/1_mb",
            "value": 2129728,
            "range": "± 35767",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}