import React from "react";
import {Card, Placeholder} from "react-bootstrap";

const SkeletonExtensionCard: React.FC = () => {
    return <Card style={{
        margin: "10px 0",
    }} className="flex-row">
        <Placeholder as="div" animation="glow" style={{ height: '200px', width: '200px', }}/>
        <Card.Body>
            <Placeholder as={Card.Title} animation="glow">
                <Placeholder xs={6} />
            </Placeholder>

            <Placeholder as={Card.Text} animation="glow">
                <Placeholder xs={7} />  <Placeholder xs={4} />{' '}
                <Placeholder xs={9} />{' '}
                <Placeholder xs={10} />
            </Placeholder>
            <Placeholder.Button variant="primary" xs={6} />
        </Card.Body>
    </Card>
}

export default SkeletonExtensionCard