import React from 'react';
import styled from 'styled-components';

// CSS in JS zone
const DashboardWrapper = styled.div`
  display: grid;
    grid-template-columns: 1fr 1fr;
 gap: 20px;
    .card {
background: white;
      border-radius:8px;
    }
`;

export const Dashboard: React.FC = () => {
    return (
        <DashboardWrapper>
            <div className="card">Stats</div>
        </DashboardWrapper>
    );
};
