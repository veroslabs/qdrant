"""Offline contracts for the OpenAPI request/response validation adapter."""
import json

import pytest
import requests
from jsonschema_rs import ValidationError
from schemathesis.core.failures import Failure, FailureGroup

from .helpers import helpers


def response(payload, content_type='application/json'):
    value = requests.Response()
    value.status_code = 200
    value.headers['Content-Type'] = content_type
    value._content = payload if isinstance(payload, bytes) else json.dumps(payload).encode()
    from urllib3.response import HTTPResponse
    value.raw = HTTPResponse(headers={'Content-Type': content_type})
    value.request = requests.Request('GET', 'http://localhost:6333/collections').prepare()
    return value


def test_valid_collection_response(monkeypatch):
    expected = response({'result': {'collections': []}, 'status': 'ok', 'time': 0.0})
    monkeypatch.setattr(helpers.requests, 'get', lambda **kwargs: expected)
    assert helpers.request_with_validation('/collections', 'GET') is expected


def test_invalid_response_is_rejected(monkeypatch):
    monkeypatch.setattr(helpers.requests, 'get', lambda **kwargs: response({'result': 123}))
    with pytest.warns(UserWarning, match='Failed validation'):
        with pytest.raises((Failure, FailureGroup)):
            helpers.request_with_validation('/collections', 'GET')


def test_undocumented_json_media_type_is_rejected(monkeypatch):
    monkeypatch.setattr(helpers.requests, 'get',
                        lambda **kwargs: response({'result': 123}, 'application/problem+json'))
    with pytest.warns(UserWarning, match='Failed validation'):
        with pytest.raises((Failure, FailureGroup)):
            helpers.request_with_validation('/collections', 'GET')


@pytest.mark.parametrize('api,params,media_type,body', [
    ('/metrics', {}, 'text/plain', b'app_info 1\n'),
    ('/snapshots/{snapshot_name}', {'snapshot_name': 'owned'},
     'application/octet-stream', b'\x00\xffsnapshot'),
])
def test_documented_non_json_responses_remain_valid(monkeypatch, api, params, media_type, body):
    expected = response(body, media_type)
    monkeypatch.setattr(helpers.requests, 'get', lambda **kwargs: expected)
    assert helpers.request_with_validation(api, 'GET', params) is expected


def test_invalid_body_rejected_before_http(monkeypatch):
    def unexpected_http(**kwargs):
        pytest.fail('Invalid request body reached HTTP transport')
    monkeypatch.setattr(helpers.requests, 'put', unexpected_http)
    with pytest.raises(ValidationError):
        helpers.request_with_validation('/collections/{collection_name}', 'PUT',
                                        {'collection_name': 'owned'},
                                        body={'vectors': {'size': 4, 'distance': 'invalid'}})


def test_nullable_request_schema():
    operation = helpers.SCHEMA['/collections/{collection_name}']['PUT']
    parameter = next(item for item in operation.body.items if item.media_type == 'application/json')
    parameter.validate({'vectors': {'size': 4, 'distance': 'Cosine'},
                        'hnsw_config': {'on_disk': None}})


def test_required_path_and_unknown_query_rejected():
    with pytest.raises(AssertionError):
        helpers.request_with_validation('/collections/{collection_name}', 'GET')
    with pytest.raises(AssertionError):
        helpers.request_with_validation('/collections', 'GET', query_params={'unknown': True})
