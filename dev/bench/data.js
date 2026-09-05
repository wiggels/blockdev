window.BENCHMARK_DATA = {
  "lastUpdate": 1788581425112,
  "repoUrl": "https://github.com/wiggels/blockdev",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e9e7e4032bd9176e4bf0f5092a377c0e7d12d601",
          "message": "Merge pull request #2 from wiggels/ci-bench-infra\n\nci: benchmark history, perf budgets, and full workflow suite",
          "timestamp": "2026-09-04T15:31:51-05:00",
          "tree_id": "c03bca07d46f9e55b3664dabb6c56d2c6b3370e9",
          "url": "https://github.com/wiggels/blockdev/commit/e9e7e4032bd9176e4bf0f5092a377c0e7d12d601"
        },
        "date": 1788554085262,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 2683920,
            "range": "± 13743",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 2387,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 2699489,
            "range": "± 12785",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 18124,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 219584,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 87580,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 436834,
            "range": "± 5660",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 148,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 174,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_hit",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_256",
            "value": 1305,
            "range": "± 6",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "17b0ec0897347ad00392d86cdba950e3cb7b4746",
          "message": "Merge pull request #9 from wiggels/fix-bench-cache\n\nci: wipe target/criterion before benching",
          "timestamp": "2026-09-04T15:53:13-05:00",
          "tree_id": "b979a26ac02f1970daba4d0bdf5a82fd35be3f87",
          "url": "https://github.com/wiggels/blockdev/commit/17b0ec0897347ad00392d86cdba950e3cb7b4746"
        },
        "date": 1788555375404,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 2720612,
            "range": "± 40040",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 2183,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 2766495,
            "range": "± 70243",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 16054,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 189525,
            "range": "± 2800",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 80023,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 345552,
            "range": "± 1308",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 143,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 168,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_hit",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_256",
            "value": 1307,
            "range": "± 22",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6e75108f6388fcc38adaf37a93ac63ccb3188c80",
          "message": "Merge pull request #10 from wiggels/release-plz-2026-09-04T20-53-47Z\n\nchore: release v0.4.2",
          "timestamp": "2026-09-04T15:58:58-05:00",
          "tree_id": "d945e4d09f39b6566a983f65b39ddb2493be2b62",
          "url": "https://github.com/wiggels/blockdev/commit/6e75108f6388fcc38adaf37a93ac63ccb3188c80"
        },
        "date": 1788555687826,
        "tool": "cargo",
        "benches": [
          {
            "name": "get_devices/spawn_only",
            "value": 1335665,
            "range": "± 16253",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/parse_live_output",
            "value": 1403,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "get_devices/full_request",
            "value": 1340815,
            "range": "± 49817",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/small_realistic",
            "value": 9208,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_human_size",
            "value": 109695,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/large_256_disks_byte_size",
            "value": 43284,
            "range": "± 1538",
            "unit": "ns/iter"
          },
          {
            "name": "parse_lsblk/size_string_heavy_1024",
            "value": 190087,
            "range": "± 1453",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_small",
            "value": 99,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "filters/non_system_filter_small",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_hit",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_filter_256",
            "value": 663,
            "range": "± 19",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f58435a55c3f3ad496a05d7d7022e914d07d489e",
          "message": "Merge pull request #4 from wiggels/sysfs-backend\n\nfeat!: walk /sys directly, drop lsblk and JSON parsing",
          "timestamp": "2026-09-04T16:12:02-05:00",
          "tree_id": "640037a38374d1052017934fa32500140b6f2f0c",
          "url": "https://github.com/wiggels/blockdev/commit/f58435a55c3f3ad496a05d7d7022e914d07d489e"
        },
        "date": 1788556443749,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 1155301,
            "range": "± 25853",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 18753807,
            "range": "± 572561",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3393,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 104,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3279,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 694112,
            "range": "± 3866",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2910734,
            "range": "± 26322",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6fcc2a0db70c6cb739dca3f08b508eeeca51f4a6",
          "message": "Merge pull request #11 from wiggels/release-plz-2026-09-04T21-12-34Z\n\nchore: release v0.5.0",
          "timestamp": "2026-09-04T16:18:01-05:00",
          "tree_id": "500370121b8108d3589ba87b4f158b80479e387f",
          "url": "https://github.com/wiggels/blockdev/commit/6fcc2a0db70c6cb739dca3f08b508eeeca51f4a6"
        },
        "date": 1788556781598,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 1053133,
            "range": "± 13236",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 16694398,
            "range": "± 590185",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3478,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 90,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3280,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 654325,
            "range": "± 4947",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2687969,
            "range": "± 18341",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "48c5e4acf89cb54b32baf684eb5a84513c037a6d",
          "message": "Merge pull request #5 from wiggels/device-identifiers\n\nfeat: add uuid, partuuid, fstype, label, partlabel, wwn, serial, model",
          "timestamp": "2026-09-04T16:34:55-05:00",
          "tree_id": "575563690d6d80c77d0130d2c46b0e8268657fee",
          "url": "https://github.com/wiggels/blockdev/commit/48c5e4acf89cb54b32baf684eb5a84513c037a6d"
        },
        "date": 1788557796216,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 325224,
            "range": "± 10998",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 5922346,
            "range": "± 137999",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 2221,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 2276,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 220065,
            "range": "± 6693",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 1331946,
            "range": "± 54172",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cce07d74cccad560166408888865f4bd21d5c3fb",
          "message": "Merge pull request #13 from wiggels/fix-model-padding\n\nfix: trim the space padded model string from ID_MODEL_ENC",
          "timestamp": "2026-09-04T16:42:21-05:00",
          "tree_id": "4d2dabda8f2b9ec22d400f02cce08c3896f46782",
          "url": "https://github.com/wiggels/blockdev/commit/cce07d74cccad560166408888865f4bd21d5c3fb"
        },
        "date": 1788558237613,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 402603,
            "range": "± 8343",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 6976832,
            "range": "± 102118",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 2989,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 144,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 2727,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 271724,
            "range": "± 3760",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 1573462,
            "range": "± 51581",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ec79f03e68a33c4c3a5a135553d93fc89989efcb",
          "message": "Merge pull request #14 from wiggels/release-plz-2026-09-04T21-42-52Z\n\nchore: release v0.6.0",
          "timestamp": "2026-09-04T16:45:54-05:00",
          "tree_id": "1bca52f2357d573a0622ec23ed6ba5d7b39bcc3f",
          "url": "https://github.com/wiggels/blockdev/commit/ec79f03e68a33c4c3a5a135553d93fc89989efcb"
        },
        "date": 1788558459592,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 903209,
            "range": "± 2695",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 15167403,
            "range": "± 274124",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 2714,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 145,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 2577,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 583578,
            "range": "± 3515",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2269415,
            "range": "± 8170",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4a93a02ff814ad7e38551e89a6c3717d13bfe73b",
          "message": "Merge pull request #15 from wiggels/bench-same-runner-ab\n\nci: gate benches against the merge base on the same runner",
          "timestamp": "2026-09-04T16:52:31-05:00",
          "tree_id": "78caf18905f7efb4df8e4dae3f078de7f519f2c6",
          "url": "https://github.com/wiggels/blockdev/commit/4a93a02ff814ad7e38551e89a6c3717d13bfe73b"
        },
        "date": 1788558856751,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 1043961,
            "range": "± 17186",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 17496525,
            "range": "± 359710",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3723,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 165,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3522,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 701557,
            "range": "± 3263",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2705220,
            "range": "± 21535",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8ad5e2b4eaf365e59eeba34640efb78bf71ffa4a",
          "message": "Merge pull request #16 from wiggels/release-plz-2026-09-04T21-53-01Z\n\nchore: release v0.6.1",
          "timestamp": "2026-09-04T16:54:55-05:00",
          "tree_id": "1e593b837296e13ca73e711ef60cae9475adfe3d",
          "url": "https://github.com/wiggels/blockdev/commit/8ad5e2b4eaf365e59eeba34640efb78bf71ffa4a"
        },
        "date": 1788559000159,
        "tool": "cargo",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 1046681,
            "range": "± 14071",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 16707324,
            "range": "± 196139",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3721,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 165,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3551,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 699517,
            "range": "± 19348",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2655800,
            "range": "± 28555",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c686af7f793727273e21343ad588ad0bde573bd7",
          "message": "Merge pull request #17 from wiggels/bench-normalized",
          "timestamp": "2026-09-04T17:25:07-05:00",
          "tree_id": "0035c9ae3245f9893e9029cc0d148c38a432ae7f",
          "url": "https://github.com/wiggels/blockdev/commit/c686af7f793727273e21343ad588ad0bde573bd7"
        },
        "date": 1788560822215,
        "tool": "cargo",
        "benches": [
          {
            "name": "calib/cpu",
            "value": 92097,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "calib/syscall",
            "value": 110617,
            "range": "± 1746",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/16",
            "value": 1168220,
            "range": "± 6101",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 20350880,
            "range": "± 879184",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3495,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 103,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3276,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 752926,
            "range": "± 5694",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2933909,
            "range": "± 45885",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "37096c0813a62e20810a5f38631774b3b2645d05",
          "message": "Merge pull request #18 from wiggels/release-plz-2026-09-04T22-25-38Z",
          "timestamp": "2026-09-04T23:08:20-05:00",
          "tree_id": "d494d50ddf1d36dd1b2801019d2133453b13feb3",
          "url": "https://github.com/wiggels/blockdev/commit/37096c0813a62e20810a5f38631774b3b2645d05"
        },
        "date": 1788581422388,
        "tool": "cargo",
        "benches": [
          {
            "name": "calib/cpu",
            "value": 81532,
            "range": "± 2106",
            "unit": "ns/iter"
          },
          {
            "name": "calib/syscall",
            "value": 102492,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/16",
            "value": 1041727,
            "range": "± 7812",
            "unit": "ns/iter"
          },
          {
            "name": "walk/disks/256",
            "value": 16592799,
            "range": "± 477346",
            "unit": "ns/iter"
          },
          {
            "name": "filters/system_256",
            "value": 3723,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 90,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 3521,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "live/get_devices",
            "value": 699008,
            "range": "± 3334",
            "unit": "ns/iter"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 2651559,
            "range": "± 28800",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "Normalized (x calibration, lower is better)": [
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c686af7f793727273e21343ad588ad0bde573bd7",
          "message": "Merge pull request #17 from wiggels/bench-normalized",
          "timestamp": "2026-09-04T17:25:07-05:00",
          "tree_id": "0035c9ae3245f9893e9029cc0d148c38a432ae7f",
          "url": "https://github.com/wiggels/blockdev/commit/c686af7f793727273e21343ad588ad0bde573bd7"
        },
        "date": 1788560823807,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 10.5609,
            "range": "± 0.0552",
            "unit": "x calib/syscall",
            "extra": "raw 1,168,220 ns/iter, calib/syscall 110,617 ns/iter on this runner"
          },
          {
            "name": "walk/disks/256",
            "value": 183.9761,
            "range": "± 7.948",
            "unit": "x calib/syscall",
            "extra": "raw 20,350,880 ns/iter, calib/syscall 110,617 ns/iter on this runner"
          },
          {
            "name": "filters/system_256",
            "value": 0.0379,
            "range": "± 0.0012",
            "unit": "x calib/cpu",
            "extra": "raw 3,495 ns/iter, calib/cpu 92,097 ns/iter on this runner"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 0.0011,
            "range": "± 0.0",
            "unit": "x calib/cpu",
            "extra": "raw 103 ns/iter, calib/cpu 92,097 ns/iter on this runner"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 0.0356,
            "range": "± 0.001",
            "unit": "x calib/cpu",
            "extra": "raw 3,276 ns/iter, calib/cpu 92,097 ns/iter on this runner"
          },
          {
            "name": "live/get_devices",
            "value": 6.8066,
            "range": "± 0.0515",
            "unit": "x calib/syscall",
            "extra": "raw 752,926 ns/iter, calib/syscall 110,617 ns/iter on this runner"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 26.5231,
            "range": "± 0.4148",
            "unit": "x calib/syscall",
            "extra": "raw 2,933,909 ns/iter, calib/syscall 110,617 ns/iter on this runner"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "wiggels@gmail.com",
            "name": "Hunter Wigelsworth",
            "username": "wiggels"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "37096c0813a62e20810a5f38631774b3b2645d05",
          "message": "Merge pull request #18 from wiggels/release-plz-2026-09-04T22-25-38Z",
          "timestamp": "2026-09-04T23:08:20-05:00",
          "tree_id": "d494d50ddf1d36dd1b2801019d2133453b13feb3",
          "url": "https://github.com/wiggels/blockdev/commit/37096c0813a62e20810a5f38631774b3b2645d05"
        },
        "date": 1788581424509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "walk/disks/16",
            "value": 10.164,
            "range": "± 0.0762",
            "unit": "x calib/syscall",
            "extra": "raw 1,041,727 ns/iter, calib/syscall 102,492 ns/iter on this runner"
          },
          {
            "name": "walk/disks/256",
            "value": 161.8936,
            "range": "± 4.6574",
            "unit": "x calib/syscall",
            "extra": "raw 16,592,799 ns/iter, calib/syscall 102,492 ns/iter on this runner"
          },
          {
            "name": "filters/system_256",
            "value": 0.0457,
            "range": "± 0.0004",
            "unit": "x calib/cpu",
            "extra": "raw 3,723 ns/iter, calib/cpu 81,532 ns/iter on this runner"
          },
          {
            "name": "filters/find_by_name_miss_256",
            "value": 0.0011,
            "range": "± 0",
            "unit": "x calib/cpu",
            "extra": "raw 90 ns/iter, calib/cpu 81,532 ns/iter on this runner"
          },
          {
            "name": "filters/find_anywhere_miss_256",
            "value": 0.0432,
            "range": "± 0.0002",
            "unit": "x calib/cpu",
            "extra": "raw 3,521 ns/iter, calib/cpu 81,532 ns/iter on this runner"
          },
          {
            "name": "live/get_devices",
            "value": 6.8201,
            "range": "± 0.0325",
            "unit": "x calib/syscall",
            "extra": "raw 699,008 ns/iter, calib/syscall 102,492 ns/iter on this runner"
          },
          {
            "name": "live/lsblk_spawn_reference",
            "value": 25.8709,
            "range": "± 0.281",
            "unit": "x calib/syscall",
            "extra": "raw 2,651,559 ns/iter, calib/syscall 102,492 ns/iter on this runner"
          }
        ]
      }
    ]
  }
}