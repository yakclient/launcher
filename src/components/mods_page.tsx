import React from "react";
import {Card} from "react-bootstrap";

const Mods: React.FC = () => {
    return <>
        <Card style={{
            margin: "10px 0",
            maxHeight: "200px",
        }}>
            <Card.Body>
                <Card.Title as="h4" className="h5 h4-sm">
                    Not done yet...
                </Card.Title>
                <Card.Text>What are you waiting for? I told you this wasnt done.</Card.Text>

            </Card.Body>
        </Card>
    </>
}

export default Mods