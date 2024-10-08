import React, {useState} from "react";
import {Button, Card} from "react-bootstrap";
import {invoke} from "@tauri-apps/api/core";
import {ExtensionMetadata, ExtensionPointer, ExtensionState, WrappedExtension} from "@/types";

const ExtensionButton: React.FC<{
    state: ExtensionState,
    onclick: (state: ExtensionState) => void
}> = ({
          state, onclick
      }) => {
    switch (state) {
        case ExtensionState.Disabled:
            return <>
                <Button style={{margin: "0 10px"}} variant="success"
                        onClick={() => onclick(ExtensionState.Enabled)}>Enable</Button>
            </>
        case ExtensionState.Enabled:
            return <>
                <Button style={{margin: "0 10px"}} variant="warning"
                        onClick={() => onclick(ExtensionState.Disabled)}>Disable</Button>
            </>
    }
}

const ExtensionCard: React.FC<{
    onclick: (state: ExtensionState) => void,
    extension: WrappedExtension
}> = ({
         onclick, extension
}) => {
    let [state, setState] = useState(extension.state)

    return <Card style={{
        margin: "10px 0",
        maxHeight: "200px",
    }} className="flex-row">
        <Card.Img
            variant="left"
            src={extension.metadata.icon ?? ""}
            height={"200px"}
        />
        <Card.Body style={{padding: "10px"}}>
            <Card.Title as="h4" className="h5 h4-sm">
                {extension.metadata.name}
            </Card.Title>
            <Card.Text>{extension.metadata.description}</Card.Text>
            <ExtensionButton state={state} onclick={(newState) => {
                setState(newState)
                onclick(newState)
            }}/>
        </Card.Body>
    </Card>
}

export default ExtensionCard