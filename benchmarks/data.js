window.BENCHMARK_DATA = {
  "lastUpdate": 1776633044883,
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
          "id": "9401e932695b8c3dfd960475a6a4351651cf7c85",
          "message": "style: format .cargo/config.toml with taplo",
          "timestamp": "2026-04-19T17:48:02-03:00",
          "tree_id": "d3f36fc20d50932ca6701ae6af2678b3302281fd",
          "url": "https://github.com/suonengine/suon/commit/9401e932695b8c3dfd960475a6a4351651cf7c85"
        },
        "date": 1776633044659,
        "tool": "cargo",
        "benches": [
          {
            "name": "checksum/calculate/empty",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-slice/empty",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/calculate/tiny",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-slice/tiny",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/calculate/small",
            "value": 888,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-slice/small",
            "value": 888,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/calculate/medium",
            "value": 14384,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-slice/medium",
            "value": 14386,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/calculate/large",
            "value": 231311,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-slice/large",
            "value": 232301,
            "range": "± 1149",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/from-vec/small",
            "value": 904,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "checksum/display/large",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/get/256",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/contains/256",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/from_iter/256",
            "value": 3309,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/get/4096",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/contains/4096",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/from_iter/4096",
            "value": 52328,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/get/16384",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/contains/16384",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "chunk/from_iter/16384",
            "value": 209754,
            "range": "± 670",
            "unit": "ns/iter"
          },
          {
            "name": "database/init_table",
            "value": 17463,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert_table",
            "value": 18172,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "database/system_param_read_write",
            "value": 290277,
            "range": "± 6703",
            "unit": "ns/iter"
          },
          {
            "name": "database/overwrite_existing/1",
            "value": 17588,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "database/overwrite_existing/64",
            "value": 17613,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "database/overwrite_existing/4096",
            "value": 17449,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "lua/exec_simple_snippet",
            "value": 190,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "lua/component_get",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "lua/component_set",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "lua/call_hook",
            "value": 810,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "lua/query_entities/1",
            "value": 4425,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "lua/query_entities/10",
            "value": 22234,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "lua/query_entities/100",
            "value": 222533,
            "range": "± 1964",
            "unit": "ns/iter"
          },
          {
            "name": "lua/query_entities/1000",
            "value": 2103826,
            "range": "± 17186",
            "unit": "ns/iter"
          },
          {
            "name": "lua/lua_exec_command_flush",
            "value": 3206,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "lua/lua_hook_command_flush",
            "value": 3930,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "lua/registry_register_component",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "macros/derive_table_like/unit_struct",
            "value": 4047,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "macros/derive_table_like/tuple_struct",
            "value": 7871,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "macros/derive_table_like/named_struct",
            "value": 10511,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "macros/derive_table_like/lifetime_generic",
            "value": 10077,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "macros/derive_table_like/parse_quote_input",
            "value": 10738,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "movement/direction_math/add_then_sub/North",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/direction_math/add_then_sub/NorthEast",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/direction_math/add_then_sub/East",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/direction_math/add_then_sub/SouthWest",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/push_pop/2",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/clear/2",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/push_pop/16",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/clear/16",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/push_pop/128",
            "value": 231,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "movement/path/clear/128",
            "value": 251,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "network/plugin_setup",
            "value": 83455,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "network/runtime/limiter_acquire_release",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "network/runtime/checksum_mode_display/1",
            "value": 59,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "network/runtime/checksum_mode_display/32",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "network/runtime/checksum_mode_display/256",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "network/runtime/packet_policy_default",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "position/btree_insert/128",
            "value": 2880,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "position/hash_insert_previous_position/128",
            "value": 6341,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "position/btree_insert/1024",
            "value": 28048,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "position/hash_insert_previous_position/1024",
            "value": 50833,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "position/btree_insert/4096",
            "value": 94433,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "position/hash_insert_previous_position/4096",
            "value": 82101,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "position/floor_ord_sort",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "position/previous_floor_hash",
            "value": 1132,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_client/decode_sequence/5",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_client/decode_sequence/13",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_client/decode_sequence/24",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_client/keep_alive_decode",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/encode_with_kind",
            "value": 72,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/keep_alive_encode",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/challenge_encode",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/encoder_roundtrip/8",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/encoder_roundtrip/64",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "protocol_server/encoder_roundtrip/512",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_millis/1234",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_millis/50000",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_millis/999999",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_secs/42",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_secs/600",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_serialize/as_secs/3600",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_millis/17",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_millis/18",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_millis/19",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_secs/15",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_secs/16",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "serde/duration_deserialize/as_secs/17",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "task/add_background_systems/update",
            "value": 71548,
            "range": "± 709",
            "unit": "ns/iter"
          },
          {
            "name": "task/add_background_systems/schedule/update",
            "value": 73736,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "task/add_background_systems/schedule/post_update",
            "value": 73391,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/expand_key",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/0",
            "value": 197,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/5",
            "value": 175,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/14",
            "value": 250,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/64",
            "value": 1046,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/encrypt/256",
            "value": 3548,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/0",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/5",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/14",
            "value": 228,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/64",
            "value": 989,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "xtea/decrypt/256",
            "value": 3552,
            "range": "± 5",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}