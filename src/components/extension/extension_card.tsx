import React, {useState} from "react";
import {Badge, Button, Card, Stack} from "react-bootstrap";
import {ExtensionState, WrappedExtension} from "@/types";
import defaultExtensionImg from "../../../public/icons/extension_icon.png"
import Image from "next/image";

export const ExtensionButton: React.FC<{
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

    const maxDescLength = 200

    let description = extension.metadata.description
    if (description.length > maxDescLength) {
        description = description.substring(0, maxDescLength - 3) + "..."
    }

    return <Card style={{
        margin: "10px 0",
        // maxHeight: "200px",
    }} className="flex-row">


        <Card.Body style={{padding: "10px"}}>

            <Stack direction="horizontal" gap={3}>
                <Image
                    alt={"OK?"}
                    src={extension.metadata.icon ?? defaultExtensionImg}
                    height={70}
                />
                <div>
                    <Card.Title as="h1" className="h1 h1-sm">
                        {extension.metadata.name}
                    </Card.Title>
                    <Card.Text>
                        By {extension.metadata.developers.join(", ")}
                        <Badge pill bg="secondary" style={{
                            marginLeft: "10px"
                        }}>
                            v{extension.pointer.descriptor.split(":")[2]}
                        </Badge>
                    </Card.Text>
                </div>
            </Stack>

            <Card.Text style={{
                margin: "25px 0"
            }}>{description}</Card.Text>
            <ExtensionButton state={state} onclick={(newState) => {
                setState(newState)
                onclick(newState)
            }}/>
        </Card.Body>
    </Card>
}

export default ExtensionCard