/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use anyhow::Context;
use buck2_cli_proto::new_generic::NewGenericRequest;
use buck2_cli_proto::new_generic::NewGenericResponse;
use buck2_server_ctx::late_bindings::OTHER_SERVER_COMMANDS;
use buck2_server_ctx::partial_result_dispatcher::NoPartialResult;
use buck2_server_ctx::partial_result_dispatcher::PartialResultDispatcher;

use crate::ctx::ServerCommandContext;
use crate::materialize::materialize_command;

pub(crate) async fn new_generic_command(
    context: &ServerCommandContext<'_>,
    req: buck2_cli_proto::NewGenericRequestMessage,
    partial_result_dispatcher: PartialResultDispatcher<NoPartialResult>,
) -> anyhow::Result<buck2_cli_proto::NewGenericResponseMessage> {
    let req = req.new_generic_request;
    let req: NewGenericRequest =
        serde_json::from_str(&req).context("Could not deserialize `NewGenericRequest`")?;
    let resp = match req {
        NewGenericRequest::Materialize(m) => {
            NewGenericResponse::Materialize(materialize_command(context, m).await?)
        }
        NewGenericRequest::Complete(e) => NewGenericResponse::Complete(
            OTHER_SERVER_COMMANDS
                .get()?
                .complete(context, partial_result_dispatcher, e)
                .await?,
        ),
        NewGenericRequest::DebugEval(e) => NewGenericResponse::DebugEval(
            OTHER_SERVER_COMMANDS.get()?.debug_eval(context, e).await?,
        ),
        NewGenericRequest::Explain(m) => NewGenericResponse::Explain(
            OTHER_SERVER_COMMANDS
                .get()?
                .explain(context, partial_result_dispatcher, m)
                .await?,
        ),
        NewGenericRequest::ExpandExternalCell(e) => NewGenericResponse::ExpandExternalCell(
            OTHER_SERVER_COMMANDS
                .get()?
                .expand_external_cell(context, partial_result_dispatcher, e)
                .await?,
        ),
    };
    let resp = serde_json::to_string(&resp).context("Could not serialize `NewGenericResponse`")?;
    Ok(buck2_cli_proto::NewGenericResponseMessage {
        new_generic_response: resp,
    })
}
