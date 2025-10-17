import fastjson


def test_round_trip():
    payload = {"message": "hello"}
    dumped = fastjson.dumps(payload)
    loaded = fastjson.loads(dumped)
    assert loaded == dumped
