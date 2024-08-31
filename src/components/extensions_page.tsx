import React, {useState} from "react";
import {Button, Card, Form} from "react-bootstrap";

enum ExtensionState {
    Enabled,
    Disabled,
    NotInstalled,
}

type Extension = {
    id: number,
    title: string,
    image: string,
    description: string,
    state: ExtensionState
}

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
                <Button style={{margin: "0 10px"}} variant="danger"
                        onClick={() => onclick(ExtensionState.NotInstalled)}>Uninstall</Button>
            </>
        case ExtensionState.Enabled:
            return <>
                <Button style={{margin: "0 10px"}} variant="warning"
                        onClick={() => onclick(ExtensionState.Disabled)}>Disable</Button>
                <Button style={{margin: "0 10px"}} variant="danger"
                        onClick={() => onclick(ExtensionState.NotInstalled)}>Uninstall</Button>
            </>
        case ExtensionState.NotInstalled:
            return <>
                <Button style={{margin: "0 10px"}} variant="success"
                        onClick={() => onclick(ExtensionState.Enabled)}>Install</Button>
            </>
    }
}

const Extensions: React.FC = () => {
    const [extensions, setExtensions] = useState<Extension[]>([
        {
            id: 0,
            title: "Steam powered",
            image: "https://media.forgecdn.net/avatars/thumbnails/1005/598/256/256/638527009420858237.png",
            description: "Steam Powered is a technology mod adding Steampunk styled machines, mobs, items, armor and more.",
            state: ExtensionState.Enabled
        },
        {
            id: 0,
            title: "Steam powered",
            image: "https://media.forgecdn.net/avatars/thumbnails/1005/598/256/256/638527009420858237.png",
            description: "Steam Powered is a technology mod adding Steampunk styled machines, mobs, items, armor and more.",
            state: ExtensionState.Enabled
        }
    ])

    const updateExtension = (index: number, updatedFields: Partial<Extension>) => {
        setExtensions((prevExtensions) =>
            prevExtensions.map((extension, i) =>
                i === index ? {...extension, ...updatedFields} : extension
            )
        );
    };

    return <>
        <form>
            <Form.Label htmlFor="inputPassword5">Search</Form.Label>
            <Form.Control/>
            <Form.Text muted>
                Changes will take place on launch (installing, etc)
            </Form.Text>
        </form>
        {extensions.map((extension, index) => {
            return <Card style={{
                margin: "10px 0",
                maxHeight: "200px",
            }} className="flex-row" key={index}>
                <Card.Img
                    variant="left"
                    src={extension.image}
                    height={"200px"}
                />
                <Card.Body style={{
                    padding: "10px"
                }}>
                    <Card.Title as="h4" className="h5 h4-sm">
                        {extension.title}
                    </Card.Title>
                    <Card.Text>{extension.description}</Card.Text>

                    <ExtensionButton state={extension.state} onclick={(state) => {
                        updateExtension(index, {
                            state: state,
                        })
                    }}/>
                </Card.Body>
            </Card>
        })}
    </>
}

export default Extensions;